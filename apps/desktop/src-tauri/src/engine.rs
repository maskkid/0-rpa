//! RPA Engine subprocess management.

use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;

/// Engine process handle.
pub struct EngineProcess {
    child: Arc<Mutex<Option<tokio::process::Child>>>,
    port: u16,
}

impl EngineProcess {
    /// Start the engine subprocess.
    pub async fn start(binary_path: &str) -> anyhow::Result<Self> {
        let port = find_available_port()?;

        let child = Command::new(binary_path)
            .env("RPA_GRPC_PORT", port.to_string())
            .env("RUST_LOG", "info")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        tracing::info!("Engine subprocess started on port {}", port);

        // Wait for engine to be ready
        wait_for_ready(port).await?;

        Ok(Self {
            child: Arc::new(Mutex::new(Some(child))),
            port,
        })
    }

    /// Get the gRPC URL for this engine.
    pub fn grpc_url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }

    /// Stop the engine subprocess.
    pub async fn stop(&self) {
        let mut guard = self.child.lock().await;
        if let Some(mut child) = guard.take() {
            let _ = child.kill().await;
            tracing::info!("Engine subprocess stopped");
        }
    }
}

/// Find an available port on localhost.
fn find_available_port() -> anyhow::Result<u16> {
    // Simple approach: try to bind to port 0, which assigns an available port
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    Ok(addr.port())
}

/// Wait for the engine to be ready by polling a health endpoint.
async fn wait_for_ready(port: u16) -> anyhow::Result<()> {
    let url = format!("http://localhost:{}/health", port);
    let client = reqwest::Client::new();

    for i in 0..30 {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!("Engine is ready at {}", url);
                return Ok(());
            }
            _ => {
                if i % 10 == 0 {
                    tracing::debug!("Waiting for engine to be ready...");
                }
            }
        }
    }

    anyhow::bail!("Engine failed to become ready within timeout")
}

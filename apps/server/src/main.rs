//! RPA Server - Main server binary.

mod config;

use anyhow::Result;
use rpa_engine::Vm;
use rpa_perception::mock::MockPerceptor;
use rpa_action::mock::MockActor;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("RPA Server starting...");

    // Load configuration
    let config = match std::env::var("RPA_CONFIG") {
        Ok(path) => config::Config::from_file(&path)?,
        Err(_) => {
            info!("No config file specified (RPA_CONFIG not set), using defaults");
            config::Config::default()
        }
    };

    info!(
        http_addr = %config.server.http_addr,
        grpc_addr = %config.server.grpc_addr,
        "Server configuration loaded"
    );

    // Build the VM with components
    let _vm = build_vm(&config).await?;

    info!("VM initialized with mock components");
    info!("Server ready (HTTP: {}, gRPC: {})", config.server.http_addr, config.server.grpc_addr);

    // TODO: Start HTTP and gRPC servers
    // For now, just run forever since this is a stub implementation
    tokio::signal::ctrl_c().await?;

    info!("Shutting down...");
    Ok(())
}

/// Build the VM with all registered components.
async fn build_vm(config: &config::Config) -> Result<Vm> {
    let vm_config: rpa_engine::VmConfig = (&config.engine).into();

    // Build with mock components for now
    // In a real implementation, this would wire up actual WindowsPerceptor,
    // WindowsActor, WindowsOcrEngine, etc. on Windows hosts
    let vm = Vm::new(vm_config)
        .with_perceptor(
            rpa_engine::finder::StrategyType::UIA,
            MockPerceptor::new("mock"),
        )
        .with_actor(MockActor::new())
        .with_debug_config(config.debug.clone());

    Ok(vm)
}

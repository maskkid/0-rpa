//! Engine subprocess management commands.

use serde::{Deserialize, Serialize};
use tauri::State;
use crate::{AppState, engine, grpc_client};

/// Task status model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatus {
    pub task_id: String,
    pub status: String,
    pub progress: f64,
}

/// Start the RPA Engine as a subprocess.
#[tauri::command]
pub async fn start_engine(
    binary_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    tracing::info!("Starting engine from: {}", binary_path);

    let engine_process = engine::EngineProcess::start(&binary_path)
        .await
        .map_err(|e| format!("Failed to start engine: {}", e))?;

    let url = engine_process.grpc_url();

    // Store in state
    {
        let mut engine_guard = state.engine.lock().await;
        *engine_guard = Some(engine_process);
    }

    // Connect gRPC client
    let client = grpc_client::EngineClient::connect(&url)
        .await
        .map_err(|e| format!("Failed to connect gRPC client: {}", e))?;

    {
        let mut client_guard = state.engine_client.lock().await;
        *client_guard = Some(client);
    }

    Ok(url)
}

/// Stop the RPA Engine subprocess.
#[tauri::command]
pub async fn stop_engine(state: State<'_, AppState>) -> Result<(), String> {
    tracing::info!("Stopping engine");

    // Stop gRPC client
    {
        let mut client_guard = state.engine_client.lock().await;
        *client_guard = None;
    }

    // Stop subprocess
    {
        let mut engine_guard = state.engine.lock().await;
        if let Some(mut process) = engine_guard.take() {
            process.stop().await;
        }
    }

    Ok(())
}

/// Execute a workflow via the engine.
#[tauri::command]
pub async fn execute_workflow(
    script: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut client_guard = state.engine_client.lock().await;
    let client = client_guard
        .as_mut()
        .ok_or("Engine not started. Call start_engine first.")?;

    let task_id = client
        .execute_workflow(&script)
        .await
        .map_err(|e| format!("Execution failed: {}", e))?;

    Ok(task_id)
}

/// Get task status from the engine.
#[tauri::command]
pub async fn get_task_status(
    task_id: String,
    state: State<'_, AppState>,
) -> Result<TaskStatus, String> {
    let mut client_guard = state.engine_client.lock().await;
    let client = client_guard
        .as_mut()
        .ok_or("Engine not started. Call start_engine first.")?;

    let status = client
        .get_task_status(&task_id)
        .await
        .map_err(|e| format!("Failed to get status: {}", e))?;

    Ok(status)
}

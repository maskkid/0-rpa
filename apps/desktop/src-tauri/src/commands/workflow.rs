//! Workflow management commands.

use serde::{Deserialize, Serialize};
use tauri::State;

/// Workflow entity model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: i32,
    pub plugin_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub script: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_run_at: Option<String>,
    pub run_count: i32,
}

/// List workflows, optionally filtered by plugin_id.
#[tauri::command]
pub async fn list_workflows(
    plugin_id: Option<i32>,
    state: State<'_, crate::AppState>,
) -> Result<Vec<Workflow>, String> {
    // TODO: Implement SeaORM query
    tracing::info!("Listing workflows, plugin_id: {:?}", plugin_id);
    Ok(vec![])
}

/// Get a single workflow by ID.
#[tauri::command]
pub async fn get_workflow(
    id: i32,
    state: State<'_, crate::AppState>,
) -> Result<Workflow, String> {
    // TODO: Implement SeaORM query
    tracing::info!("Getting workflow: {}", id);
    Err("Not implemented".into())
}

/// Save (update) a workflow's script.
#[tauri::command]
pub async fn save_workflow(
    id: i32,
    script: String,
    state: State<'_, crate::AppState>,
) -> Result<(), String> {
    // TODO: Implement SeaORM update
    tracing::info!("Saving workflow: {}", id);
    Err("Not implemented".into())
}

/// Delete a workflow.
#[tauri::command]
pub async fn delete_workflow(id: i32, state: State<'_, crate::AppState>) -> Result<(), String> {
    // TODO: Implement SeaORM deletion
    tracing::info!("Deleting workflow: {}", id);
    Err("Not implemented".into())
}

/// Create a new workflow.
#[tauri::command]
pub async fn create_workflow(
    plugin_id: i32,
    name: String,
    state: State<'_, crate::AppState>,
) -> Result<Workflow, String> {
    // TODO: Implement SeaORM insertion
    tracing::info!("Creating workflow: {} in plugin: {}", name, plugin_id);
    Err("Not implemented".into())
}

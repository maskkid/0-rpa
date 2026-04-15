//! Plugin management commands.

use serde::{Deserialize, Serialize};
use tauri::State;

/// Plugin entity model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: i32,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub zip_path: String,
    pub installed_at: String,
    pub is_enabled: bool,
}

/// List all plugins.
#[tauri::command]
pub async fn list_plugins(state: State<'_, crate::AppState>) -> Result<Vec<Plugin>, String> {
    // Stub: return empty list for now
    // TODO: Implement SeaORM query
    Ok(vec![])
}

/// Import a plugin from a ZIP file.
#[tauri::command]
pub async fn import_plugin(
    zip_path: String,
    state: State<'_, crate::AppState>,
) -> Result<Plugin, String> {
    // TODO: Implement ZIP parsing and database insertion
    tracing::info!("Importing plugin from: {}", zip_path);
    Err("Not implemented".into())
}

/// Export a plugin to a ZIP file.
#[tauri::command]
pub async fn export_plugin(
    id: i32,
    output_path: String,
    state: State<'_, crate::AppState>,
) -> Result<(), String> {
    // TODO: Implement database query and ZIP creation
    tracing::info!("Exporting plugin {} to: {}", id, output_path);
    Err("Not implemented".into())
}

/// Delete a plugin.
#[tauri::command]
pub async fn delete_plugin(id: i32, state: State<'_, crate::AppState>) -> Result<(), String> {
    // TODO: Implement database deletion
    tracing::info!("Deleting plugin: {}", id);
    Err("Not implemented".into())
}

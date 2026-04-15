//! Settings management commands.

use serde::{Deserialize, Serialize};
use tauri::State;
use crate::AppState;

/// Settings model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub database_url: String,
    pub engine_binary_path: String,
    pub engine_port: u16,
    pub engine_timeout_ms: u64,
    pub debug_enabled: bool,
    pub debug_highlight: bool,
    pub debug_highlight_duration_ms: u64,
    pub debug_screenshot_on_step: bool,
    pub debug_screenshot_dir: String,
    pub debug_slow_motion_ms: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            database_url: crate::storage::get_default_url(),
            engine_binary_path: String::new(),
            engine_port: 50051,
            engine_timeout_ms: 30000,
            debug_enabled: false,
            debug_highlight: true,
            debug_highlight_duration_ms: 500,
            debug_screenshot_on_step: false,
            debug_screenshot_dir: "./debug_screenshots".into(),
            debug_slow_motion_ms: 0,
        }
    }
}

/// Get all settings.
#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    // TODO: Load from settings table in database
    // For now, return defaults
    Ok(Settings::default())
}

/// Save settings.
#[tauri::command]
pub async fn save_settings(
    settings: Settings,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // TODO: Persist to settings table in database
    tracing::info!("Saving settings: {:?}", settings);
    Ok(())
}

/// Test a database connection.
#[tauri::command]
pub async fn test_database_connection(url: String) -> Result<bool, String> {
    // Try to connect to the database
    match crate::storage::connect(&url).await {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("Connection failed: {}", e)),
    }
}

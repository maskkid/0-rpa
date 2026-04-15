//! Debug visualization commands.

use serde::{Deserialize, Serialize};
use tauri::State;
use std::path::PathBuf;

/// Debug event model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugEvent {
    pub timestamp: String,
    pub event_type: String,
    pub bounds: Option<serde_json::Value>,
    pub label: Option<String>,
    pub color: Option<String>,
    pub duration_ms: Option<u64>,
    pub screenshot_path: Option<String>,
}

/// Get debug events for a task.
#[tauri::command]
pub async fn get_debug_events(
    task_id: i32,
    state: State<'_, crate::AppState>,
) -> Result<Vec<DebugEvent>, String> {
    // TODO: Implement debug event retrieval from task_history
    tracing::info!("Getting debug events for task: {}", task_id);
    Ok(vec![])
}

/// Get a debug screenshot as base64 or file URL.
#[tauri::command]
pub async fn get_debug_screenshot(
    path: String,
    state: State<'_, crate::AppState>,
) -> Result<String, String> {
    // TODO: Implement screenshot loading and base64 encoding
    tracing::info!("Getting debug screenshot: {}", path);

    let path = PathBuf::from(&path);
    if !path.exists() {
        return Err("Screenshot not found".into());
    }

    let data = tokio::fs::read(&path)
        .await
        .map_err(|e| format!("Failed to read screenshot: {}", e))?;

    let base64 = base64_encode(&data);
    Ok(format!("data:image/png;base64,{}", base64))
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(CHARS[b0 >> 2] as char);
        result.push(CHARS[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

        if chunk.len() > 1 {
            result.push(CHARS[((b1 & 0x0F) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARS[b2 & 0x3F] as char);
        } else {
            result.push('=');
        }
    }

    result
}

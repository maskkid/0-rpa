//! RPA Desktop Library - Tauri application entry point.

pub mod commands;
pub mod engine;
pub mod grpc_client;
pub mod storage;

use std::sync::Arc;
use storage::connection::{connect, get_default_url, DbPool};
use tauri::Manager;
use tokio::sync::Mutex;

/// Application state shared across all Tauri commands.
pub struct AppState {
    pub db: DbPool,
    pub engine: Arc<Mutex<Option<engine::EngineProcess>>>,
    pub engine_client: Arc<Mutex<Option<grpc_client::EngineClient>>>,
}

impl Default for AppState {
    fn default() -> Self {
        let db = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(connect(&get_default_url()))
            .expect("Failed to connect to database");

        Self {
            db,
            engine: Arc::new(Mutex::new(None)),
            engine_client: Arc::new(Mutex::new(None)),
        }
    }
}

/// Run the Tauri application.
pub fn run() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("RPA Desktop starting...");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::plugin::list_plugins,
            commands::plugin::import_plugin,
            commands::plugin::export_plugin,
            commands::plugin::delete_plugin,
            commands::workflow::list_workflows,
            commands::workflow::get_workflow,
            commands::workflow::save_workflow,
            commands::workflow::delete_workflow,
            commands::workflow::create_workflow,
            commands::task::list_tasks,
            commands::task::get_task_logs,
            commands::task::stream_task_events,
            commands::debug::get_debug_events,
            commands::debug::get_debug_screenshot,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::test_database_connection,
            commands::engine::start_engine,
            commands::engine::stop_engine,
            commands::engine::execute_workflow,
            commands::engine::get_task_status,
        ])
        .setup(|app| {
            tracing::info!("RPA Desktop application setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

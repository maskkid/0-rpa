//! Database connection management with SeaORM.

use sea_orm::{Database, DatabaseConnection, DbErr};
use std::sync::Arc;

/// Database connection pool type alias.
pub type DbPool = Arc<DatabaseConnection>;

/// Connect to a database using SeaORM.
pub async fn connect(database_url: &str) -> Result<DbPool, DbErr> {
    let db = Database::connect(database_url).await?;
    Ok(Arc::new(db))
}

/// Get the default SQLite database URL.
pub fn get_default_url() -> String {
    let app_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("rpa-desktop");

    // Ensure directory exists
    std::fs::create_dir_all(&app_dir).ok();

    let db_path = app_dir.join("data.db");
    format!("sqlite:{}?mode=rwc", db_path.display())
}

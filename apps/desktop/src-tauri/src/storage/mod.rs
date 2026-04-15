//! Storage layer - SeaORM database entities, migrations, and connection management.

pub mod connection;
pub mod entities;
pub mod migration;

pub use connection::{connect, get_default_url, DbPool};
//! Database module for OT common
//!
//! Provides database entities and connection utilities.

pub mod entities;

use anyhow::{Context, Result};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;
use tracing::info;

/// Connect to the database with optimized settings
pub async fn connect(database_url: &str) -> Result<DatabaseConnection> {
    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(10))
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .sqlx_logging(false);

    info!("Connecting to database...");
    let db = Database::connect(opt)
        .await
        .context("Failed to connect to database")?;
    info!("Database connection established");

    Ok(db)
}

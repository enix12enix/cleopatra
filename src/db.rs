// src/db.rs
// Initialize database configuration here

use sqlx::{sqlite::SqlitePool, sqlite::SqlitePoolOptions, Result};
use crate::config::Config;

pub async fn init_db(config: &Config) -> Result<SqlitePool> {
    // Clone the values we need for the closure
    let wal_enabled = config.database.wal;
    let wal_autocheckpoint = config.database.wal_autocheckpoint;
    
    let pool = SqlitePoolOptions::new()
        .max_connections(config.database.max_connections)
        .after_connect(move |conn, _meta| {
            Box::pin(async move {
                // Configure WAL settings if enabled
                if wal_enabled {
                    // Enable WAL mode
                    sqlx::query("PRAGMA journal_mode = WAL")
                        .execute(&mut *conn)
                        .await?;
                    
                    // Set autocheckpoint interval
                    sqlx::query(&format!("PRAGMA wal_autocheckpoint = {}", wal_autocheckpoint))
                        .execute(&mut *conn)
                        .await?;
                    
                    // Set synchronous mode to NORMAL for better performance with WAL
                    sqlx::query("PRAGMA synchronous = NORMAL")
                        .execute(&mut *conn)
                        .await?;
                }
                Ok(())
            })
        })
        .connect(&config.database.url)
        .await?;

    sqlx::query(include_str!("../migrations/cleopatra.sql"))
        .execute(&pool)
        .await?;

    Ok(pool)
}

/// Perform a WAL checkpoint with the specified mode
/// This function should be called manually when needed based on application requirements
/// Valid modes: "PASSIVE", "FULL", "RESTART", "TRUNCATE"
pub async fn perform_checkpoint(pool: &SqlitePool, mode: &str) -> Result<()> {
    let mode_upper = mode.to_uppercase();
    let valid_modes = ["PASSIVE", "FULL", "RESTART", "TRUNCATE"];
    
    if !valid_modes.contains(&mode_upper.as_str()) {
        return Err(sqlx::Error::Configuration(
            format!("Invalid checkpoint mode: {}. Valid modes are: {:?}", mode, valid_modes).into()
        ).into());
    }
    
    let query = format!("PRAGMA wal_checkpoint({})", mode_upper);
    sqlx::query(&query)
        .execute(pool)
        .await?;
        
    Ok(())
}

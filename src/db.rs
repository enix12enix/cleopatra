// src/db.rs
// Initialize database configuration here

use sqlx::{sqlite::SqlitePool, Result};
use crate::config::Config;

pub async fn init_db(config: &Config) -> Result<SqlitePool> {
    let pool = SqlitePool::connect(&config.database.url).await?;
    sqlx::query(include_str!("../migrations/cleopatra.sql"))
        .execute(&pool)
        .await?;
    Ok(pool)
}
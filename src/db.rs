// src/db.rs
// Initialize database configuration here

use sqlx::{sqlite::SqlitePool, Result};

pub async fn init_db() -> Result<SqlitePool> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    sqlx::query(include_str!("../migrations/cleopatra.sql"))
        .execute(&pool)
        .await?;
    Ok(pool)
}
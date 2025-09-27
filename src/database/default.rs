// src/db.rs
// Main database for test result 

use sqlx::{sqlite::SqlitePool, sqlite::SqlitePoolOptions, Result, SqliteConnection};
use crate::config::Config;
use crate::models::{CreateTestResult, TestResult};

pub async fn init_db(config: &Config) -> Result<(SqlitePool, SqlitePool)> {
    let wal_enabled = config.database.wal;
    let wal_autocheckpoint = config.database.wal_autocheckpoint;
    
    // Main pool for application routes
    let main_pool = SqlitePoolOptions::new()
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

    // Dedicated pool for writer with only 1 connection
    let writer_pool = SqlitePoolOptions::new()
        .max_connections(1)  // Only 1 connection for the writer
        .connect(&config.database.url)
        .await?;

    // Run migrations on both pools
    sqlx::query(include_str!("../../migrations/cleopatra.sql"))
        .execute(&main_pool)
        .await?;

    Ok((main_pool, writer_pool))
}

/// Upsert (insert or update) a test result.
/// If a test result with the same (execution_id, name) exists, it will be updated and the counter incremented.
/// Otherwise, a new test result will be created with counter set to 1.
pub async fn upsert_test_result(
    conn: &mut SqliteConnection,
    payload: &CreateTestResult,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO test_result (
            execution_id, name, platform, description, status,
            execution_time, counter, log, screenshot_id, created_by, time_created
        )
        VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?, ?, ?)
        ON CONFLICT(execution_id, name) DO UPDATE SET
            platform = excluded.platform,
            description = excluded.description,
            status = excluded.status,
            execution_time = excluded.execution_time,
            counter = test_result.counter + 1,
            log = excluded.log,
            screenshot_id = excluded.screenshot_id
        "#
    )
    .bind(payload.execution_id)
    .bind(&payload.name)
    .bind(&payload.platform)
    .bind(payload.description.as_deref())
    .bind(&payload.status)
    .bind(payload.execution_time)
    .bind(&payload.log)
    .bind(&payload.screenshot_id)
    .bind(&payload.created_by.as_deref())
    .bind(payload.time_created)
    .execute(conn)
    .await?;

    Ok(())
}

/// Check if an execution exists by its ID
pub async fn check_execution_existing(
    conn: &mut SqliteConnection,
    execution_id: i64,
) -> bool {
    match sqlx::query_scalar::<_, i64>(
        "SELECT id FROM execution WHERE id = ?"
    )
    .bind(execution_id)
    .fetch_optional(conn)
    .await {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(_) => false,
    }
}

/// Update the status of a test result by its ID
pub async fn update_test_result_status(
    conn: &mut SqliteConnection,
    id: i64,
    status: &crate::models::Status,
) -> Result<TestResult> {
    let test_result = sqlx::query_as::<_, TestResult>(
        r#"
        UPDATE test_result 
        SET status = ?
        WHERE id = ?
        RETURNING *
        "#
    )
    .bind(status)
    .bind(id)
    .fetch_one(conn)
    .await?;

    Ok(test_result)
}

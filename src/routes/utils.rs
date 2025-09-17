// src/routes/utils.rs
// Utility functions for routes

use sqlx::{SqliteConnection, Result as SqlxResult};
use crate::models::TestResult;

/// Upsert (insert or update) a test result
/// If a test result with the same execution_id and name exists, it will be updated and the counter incremented
/// Otherwise, a new test result will be created with counter set to 1
pub async fn upsert_test_result(
    conn: &mut SqliteConnection,
    execution_id: i64,
    name: String,
    platform: String,
    description: Option<String>,
    status: String,
    execution_time: Option<i64>,
    log: Option<String>,
    screenshot_id: Option<i64>,
    created_by: Option<String>,
    time_created: i64,
) -> SqlxResult<TestResult> {
    // Check if test result already exists
    let existing_result: Option<TestResult> = sqlx::query_as::<_, TestResult>(
        "SELECT * FROM test_result WHERE execution_id = ? AND name = ?"
    )
    .bind(execution_id)
    .bind(&name)
    .fetch_optional(&mut *conn)
    .await?;
    
    if let Some(mut existing) = existing_result {
        // Update existing result and increment counter
        existing.counter += 1;
        sqlx::query(
            "UPDATE test_result SET platform = ?, description = ?, status = ?, execution_time = ?, counter = ?, log = ?, screenshot_id = ?, created_by = ?, time_created = ? WHERE id = ?"
        )
        .bind(&platform)
        .bind(&description)
        .bind(&status)
        .bind(execution_time)
        .bind(existing.counter)
        .bind(&log)
        .bind(&screenshot_id)
        .bind(&created_by)
        .bind(time_created)
        .bind(existing.id.unwrap())
        .execute(&mut *conn)
        .await?;
        
        Ok(existing)
    } else {
        // Create new result
        let test_result = sqlx::query_as::<_, TestResult>(
            "INSERT INTO test_result (execution_id, name, platform, description, status, execution_time, counter, log, screenshot_id, created_by, time_created) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING *"
        )
        .bind(execution_id)
        .bind(&name)
        .bind(&platform)
        .bind(&description)
        .bind(&status)
        .bind(execution_time)
        .bind(1) // counter starts at 1
        .bind(&log)
        .bind(&screenshot_id)
        .bind(&created_by)
        .bind(time_created)
        .fetch_one(&mut *conn)
        .await?;
        
        Ok(test_result)
    }
}
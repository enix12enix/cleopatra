// src/routes/utils.rs
// Utility functions for routes

use sqlx::{SqliteConnection, Result as SqlxResult};
use crate::models::{TestResult, CreateTestResult};

/// Upsert (insert or update) a test result.
/// If a test result with the same (execution_id, name) exists, it will be updated and the counter incremented.
/// Otherwise, a new test result will be created with counter set to 1.
pub async fn upsert_test_result(
    conn: &mut SqliteConnection,
    payload: &CreateTestResult,
) -> SqlxResult<TestResult> {
    let test_result = sqlx::query_as::<_, TestResult>(
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
        RETURNING *
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
    .bind(payload.created_by.as_deref())
    .bind(payload.time_created)
    .fetch_one(conn)
    .await?;

    Ok(test_result)
}

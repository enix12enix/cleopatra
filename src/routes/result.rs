// src/routes/result.rs
// Define restful test result API here

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::models::TestResult;

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .route("/api/result", post(create_test_result))
        .route("/api/result/:id", get(get_test_result))
}

async fn create_test_result(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateTestResult>,
) -> Result<(StatusCode, Json<TestResult>), (StatusCode, String)> {
    let mut conn = pool.acquire().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Check if test result already exists
    let existing_result: Option<TestResult> = sqlx::query_as::<_, TestResult>(
        "SELECT * FROM test_result WHERE execution_id = ? AND name = ?"
    )
    .bind(payload.execution_id)
    .bind(&payload.name)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let test_result = if let Some(mut existing) = existing_result {
        // Update existing result and increment counter
        existing.counter += 1;
        sqlx::query(
            "UPDATE test_result SET platform = ?, description = ?, status = ?, execution_time = ?, counter = ?, log = ?, screenshot_id = ?, created_by = ?, time_created = ? WHERE id = ?"
        )
        .bind(&payload.platform)
        .bind(&payload.description)
        .bind(&payload.status)
        .bind(payload.execution_time)
        .bind(existing.counter)
        .bind(&payload.log)
        .bind(&payload.screenshot_id)
        .bind(&payload.created_by)
        .bind(payload.time_created)
        .bind(existing.id.unwrap())
        .execute(&mut *conn)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
        existing
    } else {
        // Create new result
        sqlx::query_as::<_, TestResult>(
            "INSERT INTO test_result (execution_id, name, platform, description, status, execution_time, counter, log, screenshot_id, created_by, time_created) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING *"
        )
        .bind(payload.execution_id)
        .bind(&payload.name)
        .bind(&payload.platform)
        .bind(&payload.description)
        .bind(&payload.status)
        .bind(payload.execution_time)
        .bind(1) // counter starts at 1
        .bind(&payload.log)
        .bind(&payload.screenshot_id)
        .bind(&payload.created_by)
        .bind(payload.time_created)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };
    
    Ok((StatusCode::CREATED, Json(test_result)))
}

async fn get_test_result(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<Json<TestResult>, (StatusCode, String)> {
    let mut conn = pool.acquire().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let test_result = sqlx::query_as::<_, TestResult>(
        "SELECT * FROM test_result WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Test result not found".to_string()))?;
    
    Ok(Json(test_result))
}

#[derive(Deserialize)]
struct CreateTestResult {
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
}
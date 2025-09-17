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
use crate::routes::utils::upsert_test_result;

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
    
    let test_result = upsert_test_result(
        &mut *conn,
        payload.execution_id,
        payload.name,
        payload.platform,
        payload.description,
        payload.status,
        payload.execution_time,
        payload.log,
        payload.screenshot_id,
        payload.created_by,
        payload.time_created,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
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
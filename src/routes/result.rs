// src/routes/result.rs
// Define restful test result API here

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};

use crate::models::{AppState, TestResult, CreateTestResult, CreateTestResultResponse};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/result", post(create_test_result))
        .route("/api/result/:id", get(get_test_result))
}

async fn create_test_result(
    State(state): State<AppState>,
    Json(payload): Json<CreateTestResult>,
) -> Result<(StatusCode, Json<CreateTestResultResponse>), (StatusCode, String)> {
    // Enqueue the result to be processed by the background writer
    state.writer.enqueue(payload).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    
    // Return a response indicating the result was delivered to the queue
    let response = CreateTestResultResponse {
        status: "delivered".to_string(),
    };
    
    Ok((StatusCode::CREATED, Json(response)))
}

async fn get_test_result(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<TestResult>, (StatusCode, String)> {
    let mut conn = state.pool.acquire().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
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


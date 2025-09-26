// src/routes/result.rs
// Define restful test result API here

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, patch},
    Router,
};

use crate::db::{check_execution_existing, update_test_result_status as db_update_test_result_status};
use crate::models::{TestResult, CreateTestResult, CreateTestResultResponse, Status, UpdateStatusRequest};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/result", post(create_test_result))
        .route("/api/result/:id", get(get_test_result))
        .route("/api/result/:id/status", patch(update_test_result_status))
}

async fn create_test_result(
    State(state): State<AppState>,
    Json(payload): Json<CreateTestResult>,
) -> Result<(StatusCode, Json<CreateTestResultResponse>), (StatusCode, String)> {
    // Check if the execution exists
    {
        let mut conn = state.pool.acquire().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        if !check_execution_existing(&mut *conn, payload.execution_id).await {
            let error_message = format!("invalid execution_id, no execution is found, execution_id :: {}", payload.execution_id);
            return Err((StatusCode::BAD_REQUEST, error_message));
        }
    }

    // Enqueue the result to be processed by the background writer
    state.writer.enqueue(payload).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    
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

async fn update_test_result_status(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateStatusRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Convert the string status to the Status enum using the value_of function
    let status = Status::value_of(&payload.status)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let mut conn = state.pool.acquire().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match db_update_test_result_status(&mut *conn, id, &status).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT), // 204 No Content
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}


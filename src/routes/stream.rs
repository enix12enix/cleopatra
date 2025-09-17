// src/routes/stream.rs
// Define stream API here

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::post,
    Router,
};
use axum::body::Body;
use futures::{StreamExt, TryStreamExt};
use futures::AsyncBufReadExt;

use crate::models::{AppState, CreateTestResult, StreamResponse, FailedItem};
use crate::routes::utils::upsert_test_result;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/executions/:execution_id/results:stream", post(stream_test_results))
}

async fn stream_test_results(
    Path(execution_id): Path<i64>,
    State(state): State<AppState>,
    body: Body,
) -> Result<Json<StreamResponse>, (StatusCode, String)> {
    let mut conn = state.pool.acquire().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Convert Body to a stream of JSON values
    let stream = body
        .into_data_stream()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        .into_async_read()
        .lines();
    
    futures::pin_mut!(stream);
    
    let mut received = 0;
    let mut inserted = 0;
    let mut failed = 0;
    let mut failed_items = Vec::new();
    
    while let Some(line_result) = stream.next().await {
        received += 1;
        
        match line_result {
            Ok(line) => {
                // Parse JSON from line
                let payload: Result<CreateTestResult, _> = serde_json::from_str(&line);
                
                match payload {
                    Ok(payload) => {
                        // Validate payload
                        if !matches!(payload.status.as_str(), "P" | "F" | "I") {
                            failed += 1;
                            let raw_payload = serde_json::to_value(&payload).unwrap_or(serde_json::Value::Null);
                            failed_items.push(FailedItem {
                                test_name: payload.name.clone(),
                                error: format!("Invalid status value: {}", payload.status),
                                raw_payload,
                            });
                            continue;
                        }
                        
                        // Clone the values we need before moving them
                        let name = payload.name.clone();
                        let platform = payload.platform.clone();
                        let description = payload.description.clone();
                        let status = payload.status.clone();
                        let execution_time = payload.execution_time;
                        let log = payload.log.clone();
                        let screenshot_id = payload.screenshot_id;
                        let created_by = payload.created_by.clone();
                        let time_created = payload.time_created;
                        
                        let insert_result = upsert_test_result(
                            &mut *conn,
                            execution_id,
                            name,
                            platform,
                            description,
                            status,
                            execution_time,
                            log,
                            screenshot_id,
                            created_by,
                            time_created,
                        )
                        .await;
                        
                        match insert_result {
                            Ok(_) => inserted += 1,
                            Err(e) => {
                                failed += 1;
                                let raw_payload = serde_json::to_value(&payload).unwrap_or(serde_json::Value::Null);
                                failed_items.push(FailedItem {
                                    test_name: payload.name,
                                    error: e.to_string(),
                                    raw_payload,
                                });
                            }
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        failed_items.push(FailedItem {
                            test_name: "Unknown".to_string(),
                            error: e.to_string(),
                            raw_payload: serde_json::Value::Null,
                        });
                    }
                }
            }
            Err(e) => {
                failed += 1;
                failed_items.push(FailedItem {
                    test_name: "Unknown".to_string(),
                    error: e.to_string(),
                    raw_payload: serde_json::Value::Null,
                });
            }
        }
    }
    
    let status = if failed == 0 {
        "C" // Completed
    } else if inserted > 0 {
        "P" // Partial
    } else {
        "F" // Failed
    };
    
    let message = if failed == 0 {
        "All test results processed successfully".to_string()
    } else {
        "Some test results failed".to_string()
    };
    
    let response = StreamResponse {
        status: status.to_string(),
        message,
        execution_id,
        received,
        inserted,
        failed,
        failed_items: if failed > 0 { Some(failed_items) } else { None },
    };
    
    Ok(Json(response))
}


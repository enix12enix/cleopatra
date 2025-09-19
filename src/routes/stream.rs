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

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/executions/:execution_id/result/stream", post(stream_test_results))
}

async fn stream_test_results(
    Path(execution_id): Path<i64>,
    State(state): State<AppState>,
    body: Body,
) -> Result<Json<StreamResponse>, (StatusCode, String)> {
    // Convert Body to a stream of JSON values
    let stream = body
        .into_data_stream()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        .into_async_read()
        .lines();
    
    futures::pin_mut!(stream);
    
    let mut received = 0;
    let mut enqueued = 0;
    let mut failed = 0;
    let mut failed_items = Vec::new();
    
    while let Some(line_result) = stream.next().await {
        received += 1;
        
        match line_result {
            Ok(line) => {
                // Parse JSON from line
                let payload: Result<CreateTestResult, _> = serde_json::from_str(&line);
                
                match payload {
                    Ok(mut payload) => {
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
                        
                        // Set the execution_id from the path parameter
                        payload.execution_id = execution_id;
                        
                        // Enqueue the result to be processed by the background writer
                        match state.writer.enqueue(payload).await {
                            Ok(_) => enqueued += 1,
                            Err(e) => {
                                failed += 1;
                                failed_items.push(FailedItem {
                                    test_name: "Unknown".to_string(),
                                    error: e,
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
    } else if enqueued > 0 {
        "P" // Partial
    } else {
        "F" // Failed
    };
    
    let message = if failed == 0 {
        "All test results enqueued successfully".to_string()
    } else {
        "Some test results failed to enqueue".to_string()
    };
    
    let response = StreamResponse {
        status: status.to_string(),
        message,
        execution_id,
        received,
        inserted: enqueued, // Using inserted field to represent enqueued items
        failed,
        failed_items: if failed > 0 { Some(failed_items) } else { None },
    };
    
    Ok(Json(response))
}


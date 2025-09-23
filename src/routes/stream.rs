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

use crate::models::{CreateTestResult, StreamResponse, FailedItem};
use crate::state::AppState;

// Status constants for stream response
const STATUS_COMPLETED: &str = "C"; // Completed
const STATUS_PARTIAL: &str = "P";   // Partial
const STATUS_FAILED: &str = "F";    // Failed

fn to_status(failed: i64, enqueued: i64) -> &'static str {
    if failed == 0 {
        STATUS_COMPLETED
    } else if enqueued > 0 {
        STATUS_PARTIAL
    } else {
        STATUS_FAILED
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/executions/:execution_id/result/stream", post(stream_test_results))
}

async fn stream_test_results(
    Path(execution_id): Path<i64>,
    State(state): State<AppState>,
    body: Body,
) -> Result<Json<StreamResponse>, (StatusCode, String)> {
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
                let payload: Result<CreateTestResult, _> = CreateTestResult::from_json(&line, execution_id);
                
                match payload {
                    Ok(payload) => {
                        // Enqueue the result to be processed by the background writer
                        match state.writer.enqueue(payload).await {
                            Ok(_) => enqueued += 1,
                            Err(e) => {
                                failed += 1;
                                failed_items.push(FailedItem {
                                    error: e,
                                    raw_payload: Some(line)
                                });
                            }
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        failed_items.push(FailedItem {
                            error: e.to_string(),
                            raw_payload: Some(line)
                        });
                    }
                }
            }
            Err(e) => {
                failed += 1;
                failed_items.push(FailedItem {
                    error: e.to_string(),
                    raw_payload: None
                });
            }
        }
    }
    
    let response = StreamResponse {
        status: to_status(failed, enqueued).to_string(),
        execution_id,
        received,
        inserted: enqueued,
        failed,
        failed_items: if failed > 0 { Some(failed_items) } else { None },
    };
    
    Ok(Json(response))
}

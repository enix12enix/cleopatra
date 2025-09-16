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
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::models::TestResult;

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .route("/api/executions/:execution_id/results:stream", post(stream_test_results))
}

async fn stream_test_results(
    Path(execution_id): Path<i64>,
    State(pool): State<SqlitePool>,
    body: Body,
) -> Result<Json<StreamResponse>, (StatusCode, String)> {
    let mut conn = pool.acquire().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
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
                        
                        // Check if test result already exists
                        let existing_result: Option<TestResult> = sqlx::query_as::<_, TestResult>(
                            "SELECT * FROM test_result WHERE execution_id = ? AND name = ?"
                        )
                        .bind(execution_id)
                        .bind(&payload.name)
                        .fetch_optional(&mut *conn)
                        .await
                        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                        
                        let insert_result = if let Some(mut existing) = existing_result {
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
                        } else {
                            // Create new result
                            sqlx::query(
                                "INSERT INTO test_result (execution_id, name, platform, description, status, execution_time, counter, log, screenshot_id, created_by, time_created) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
                            )
                            .bind(execution_id)
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
                            .execute(&mut *conn)
                            .await
                        };
                        
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

#[derive(Deserialize, Serialize, Clone)]
struct CreateTestResult {
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

#[derive(Serialize)]
struct StreamResponse {
    status: String,
    message: String,
    execution_id: i64,
    received: i64,
    inserted: i64,
    failed: i64,
    failed_items: Option<Vec<FailedItem>>,
}

#[derive(Serialize)]
struct FailedItem {
    test_name: String,
    error: String,
    raw_payload: serde_json::Value,
}
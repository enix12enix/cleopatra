// src/routes/execution.rs
// Define restful execution API here

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;

use crate::models::Execution;

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .route("/api/execution", post(create_execution))
        .route("/api/executions", get(get_executions))
        .route("/api/execution/:id/result", get(get_execution_results))
}

async fn create_execution(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateExecution>,
) -> Result<(StatusCode, Json<Execution>), (StatusCode, String)> {
    let mut conn = pool.acquire().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let execution = sqlx::query_as::<_, Execution>(
        "INSERT INTO execution (name, tag, created_by, time_created) VALUES (?, ?, ?, ?) RETURNING *"
    )
    .bind(&payload.name)
    .bind(&payload.tag)
    .bind(&payload.created_by)
    .bind(payload.time_created)
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(execution)))
}

async fn get_executions(
    State(pool): State<SqlitePool>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ExecutionListResponse>, (StatusCode, String)> {
    let mut conn = pool.acquire().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(20).min(100);
    let offset: i64 = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);
    
    let mut query = "SELECT * FROM execution WHERE 1=1".to_string();
    let mut count_query = "SELECT COUNT(*) FROM execution WHERE 1=1".to_string();
    let mut bindings = Vec::new();
    
    if let Some(created_by) = params.get("created_by") {
        query.push_str(" AND created_by = ?");
        count_query.push_str(" AND created_by = ?");
        bindings.push(created_by.clone());
    }
    
    if let Some(name) = params.get("name") {
        query.push_str(" AND name LIKE ?");
        count_query.push_str(" AND name LIKE ?");
        bindings.push(format!("%{}%", name));
    }
    
    if let Some(tag) = params.get("tag") {
        query.push_str(" AND tag = ?");
        count_query.push_str(" AND tag = ?");
        bindings.push(tag.clone());
    }
    
    // Add ordering, limit and offset
    query.push_str(" ORDER BY id DESC LIMIT ? OFFSET ?");
    
    // Fetch total count
    let mut count_query_builder = sqlx::query_scalar::<_, i64>(&count_query);
    for binding in &bindings {
        count_query_builder = count_query_builder.bind(binding);
    }
    let total = count_query_builder.fetch_one(&mut *conn).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Fetch items
    let mut query_builder = sqlx::query_as::<_, Execution>(&query);
    for binding in &bindings {
        query_builder = query_builder.bind(binding);
    }
    query_builder = query_builder.bind(limit).bind(offset);
    let items = query_builder.fetch_all(&mut *conn).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let has_next = (offset + limit) < total;
    
    let response = ExecutionListResponse {
        total,
        limit,
        offset,
        has_next,
        items,
    };
    
    Ok(Json(response))
}

async fn get_execution_results(
    Path(_id): Path<i64>,
    State(_pool): State<SqlitePool>,
    Query(_params): Query<HashMap<String, String>>,
) -> Result<Json<ExecutionResultsResponse>, (StatusCode, String)> {
    // Implementation will be added later
    Err((StatusCode::NOT_IMPLEMENTED, "Not implemented".to_string()))
}

#[derive(Deserialize)]
struct CreateExecution {
    name: String,
    tag: Option<String>,
    created_by: Option<String>,
    time_created: i64,
}

#[derive(Serialize)]
struct ExecutionListResponse {
    total: i64,
    limit: i64,
    offset: i64,
    has_next: bool,
    items: Vec<Execution>,
}

#[derive(Serialize)]
struct ExecutionResultsResponse {
    execution_id: i64,
    summary: Option<Summary>,
    total: i64,
    limit: i64,
    offset: i64,
    has_next: bool,
    items: Vec<crate::models::TestResult>,
}

#[derive(Serialize)]
struct Summary {
    total: i64,
    pass: i64,
    fail: i64,
    ignor: i64,
}
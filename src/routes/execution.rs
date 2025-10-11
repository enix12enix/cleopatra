// src/routes/execution.rs
// Define restful execution API here

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::collections::HashMap;

use crate::models::{Execution, CreateExecution, ExecutionListResponse, ExecutionResultsResponse, TestResult, Summary, SuggestedItem, SuggestQuery, SuggestResponse};
use crate::state::AppState;

pub fn routes(app_state: &AppState) -> Router<AppState> {
    let mut router = Router::new()
        .route("/api/execution", post(create_execution))
        .route("/api/executions", get(get_executions))
        .route("/api/execution/:id/result", get(get_execution_results));

    // Conditionally add the suggest route based on configuration
    if app_state.config.execution_suggest.enabled {
        router = router.route("/api/executions/suggest", get(get_suggested_executions));
    }

    router
}

async fn create_execution(
    State(state): State<AppState>,
    Json(payload): Json<CreateExecution>,
) -> Result<(StatusCode, Json<Execution>), (StatusCode, String)> {
    let mut conn = state.pool.acquire().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
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

    // Add the new execution name to the prefix trie for suggestions (if enabled)
    if let Some(ref trie) = state.execution_prefix_trie {
        let mut trie_write = trie.write();
        let item = SuggestedItem {
            id: execution.id.unwrap_or(0).to_string(),
            name: execution.name.clone(),
        };
        trie_write.insert(&execution.name, item);
    }

    Ok((StatusCode::CREATED, Json(execution)))
}

async fn get_executions(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ExecutionListResponse>, (StatusCode, String)> {
    let mut conn = state.pool.acquire().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
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
        query.push_str(" AND name = ?");
        count_query.push_str(" AND name = ?");
        bindings.push(name.clone());
    }
    
    if let Some(tag) = params.get("tag") {
        query.push_str(" AND tag LIKE ?");
        count_query.push_str(" AND tag LIKE ?");
        bindings.push(format!("{}%", tag));
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
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ExecutionResultsResponse>, (StatusCode, String)> {
    let mut conn = state.pool.acquire().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(20).min(100);
    let offset: i64 = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);
    
    // Build the base query for test results
    let mut query = "SELECT * FROM test_result WHERE execution_id = ?".to_string();
    let mut count_query = "SELECT COUNT(*) FROM test_result WHERE execution_id = ?".to_string();
    let mut bindings = vec![id.to_string()];
    
    // Add filters
    if let Some(status) = params.get("status") {
        query.push_str(" AND status = ?");
        count_query.push_str(" AND status = ?");
        bindings.push(status.clone());
    }
    
    if let Some(platform) = params.get("platform") {
        query.push_str(" AND platform = ?");
        count_query.push_str(" AND platform = ?");
        bindings.push(platform.clone());
    }
    
    // Add ordering, limit and offset
    query.push_str(" ORDER BY id ASC LIMIT ? OFFSET ?");
    
    // Fetch total count
    let mut count_query_builder = sqlx::query_scalar::<_, i64>(&count_query);
    for binding in &bindings {
        count_query_builder = count_query_builder.bind(binding);
    }
    // Add limit and offset parameters for count query (they're not used but needed for binding)
    count_query_builder = count_query_builder.bind(limit).bind(offset);
    let total = count_query_builder.fetch_one(&mut *conn).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Fetch items
    let mut query_builder = sqlx::query_as::<_, TestResult>(&query);
    for binding in &bindings {
        query_builder = query_builder.bind(binding);
    }
    query_builder = query_builder.bind(limit).bind(offset);
    let items = query_builder.fetch_all(&mut *conn).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let has_next = (offset + limit) < total;
    
    // Calculate summary if requested
    let summary = if params.get("include_summary").map(|s| s.as_str()) == Some("true") {
        let pass_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_result WHERE execution_id = ? AND status = 'P'")
            .bind(id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            
        let fail_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_result WHERE execution_id = ? AND status = 'F'")
            .bind(id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            
        let ignor_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_result WHERE execution_id = ? AND status = 'I'")
            .bind(id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
        Some(Summary {
            total: pass_count + fail_count + ignor_count,
            pass: pass_count,
            fail: fail_count,
            ignor: ignor_count,
        })
    } else {
        None
    };
    
    let response = ExecutionResultsResponse {
        execution_id: id,
        summary,
        total,
        limit,
        offset,
        has_next,
        items,
    };
    
    Ok(Json(response))
}


async fn get_suggested_executions(
    State(state): State<AppState>,
    Query(params): Query<SuggestQuery>,
) -> Result<Json<SuggestResponse>, (StatusCode, String)> {
    // Check if execution suggestions are enabled
    if !state.config.execution_suggest.enabled {
        return Err((StatusCode::NOT_FOUND, "Execution suggestions are disabled".to_string()));
    }

    let query = params.query.unwrap_or_default();
    
    let min_query_len = state.config.execution_suggest.min_query_len;
    let max_candidates = state.config.execution_suggest.max_candidates;
    
    // Limit the query length to reasonable size
    if query.len() < min_query_len {
        let response = SuggestResponse {
            query,
            suggestions: vec![],
            limit: max_candidates,
        };
        return Ok(Json(response));
    }
    
    // Get the prefix trie from the application state (it should exist since we checked enabled flag)
    let trie = state.execution_prefix_trie.as_ref()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Execution prefix trie not initialized".to_string()))?;
    let trie_read = trie.read();
    let mut suggestions = trie_read.search(&query);
    
    // Truncate to max_candidates if necessary
    if suggestions.len() > max_candidates {
        suggestions.truncate(max_candidates);
    }
    
    let response = SuggestResponse {
        query,
        suggestions,
        limit: max_candidates,
    };
    
    Ok(Json(response))
}

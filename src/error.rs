use axum::{
    Json,
    body::{Body, Bytes, to_bytes},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use futures::FutureExt;

use crate::models::ErrorResponse;

/// Middleware to catch panics and error responses, converting to `ErrorResponse`.
pub async fn handle_unexpected_errors(req: Request<Body>, next: Next) -> Response {
    // Catch panics
    let result = std::panic::AssertUnwindSafe(next.run(req))
        .catch_unwind()
        .await;

    let response = match result {
        Ok(resp) => resp,
        Err(_) => {
            let body = ErrorResponse {
                error: "INTERNAL_ERROR".into(),
                message: "An unexpected error occurred".into(),
                field: None,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response();
        }
    };

    // Normalize error responses
    if response.status().is_client_error() || response.status().is_server_error() {
        let status = response.status();

        let body_bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap_or_else(|_| Bytes::new());
        let msg = String::from_utf8_lossy(&body_bytes).to_string();

        let error_type = match status {
            StatusCode::BAD_REQUEST => "BAD_REQUEST",
            StatusCode::UNAUTHORIZED => "UNAUTHORIZED",
            StatusCode::FORBIDDEN => "FORBIDDEN",
            StatusCode::NOT_FOUND => "NOT_FOUND",
            StatusCode::CONFLICT => "CONFLICT",
            StatusCode::INTERNAL_SERVER_ERROR => "INTERNAL_ERROR",
            _ if status.is_client_error() => "CLIENT_ERROR",
            _ if status.is_server_error() => "SERVER_ERROR",
            _ => "ERROR",
        };

        let body = ErrorResponse {
            error: error_type.into(),
            message: if msg.is_empty() {
                status.canonical_reason().unwrap_or("Unknown error").into()
            } else {
                msg
            },
            field: None,
        };

        return (status, Json(body)).into_response();
    }

    response
}

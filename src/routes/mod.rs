// src/routes/mod.rs
// Define axum routes here

use axum::Router;

mod execution;
mod result;
mod stream;

use crate::models::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(execution::routes())
        .merge(result::routes())
        .merge(stream::routes())
}
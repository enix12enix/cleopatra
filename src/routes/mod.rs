// src/routes/mod.rs
// Define axum routes here

use axum::Router;

mod execution;
mod result;
mod stream;

use crate::state::AppState;

pub fn routes(app_state: &AppState) -> Router<AppState> {
    Router::new()
        .merge(execution::routes(app_state))
        .merge(result::routes())
        .merge(stream::routes())
}
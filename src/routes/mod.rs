// src/routes/mod.rs
// Define axum routes here

use axum::Router;
use sqlx::SqlitePool;

mod execution;
mod result;
mod stream;
mod utils;

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .merge(execution::routes())
        .merge(result::routes())
        .merge(stream::routes())
}
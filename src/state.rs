// src/state.rs
// Application state module

use sqlx::SqlitePool;
use crate::writer::Writer;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub writer: Writer,
}
// src/state.rs
// Application state module

use std::sync::Arc;

use sqlx::SqlitePool;
use crate::{auth::AuthProvider, writer::Writer};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub writer: Writer,
    pub auth_provider: Option<Arc<AuthProvider>>,
}
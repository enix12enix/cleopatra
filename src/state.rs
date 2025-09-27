// src/state.rs
// Application state module

use std::sync::Arc;

use sqlx::SqlitePool;
use crate::auth::{self, AuthProvider};
use crate::{config, database};
use crate::daemon::writer::{DefaultWriter, Writer, WriterManager, WriterName};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub writer_manager: Arc<WriterManager>,
    pub auth_provider: Option<Arc<AuthProvider>>,
}

pub async fn init_state() -> Result<crate::state::AppState, Box<dyn std::error::Error>> {
    let config = config::Config::from_env()?;

    // init sqlite connection pool
    let (main_pool, writer_pool) = database::init_db(&config).await?;

    // init write manager
    let mut writer_manager = WriterManager::new();
    let default_writer = DefaultWriter::new(&config, writer_pool).await;
    writer_manager.insert(WriterName::Main, default_writer);

    // init auth provider
    let auth_provider = if config.auth.enabled {
        let provider = auth::AuthProvider::new(&config)?;
        Some(Arc::new(provider))
    } else {
        None
    };
    let state = AppState { 
        pool: main_pool, 
        writer_manager: Arc::new(writer_manager),
        auth_provider,
    };

    Ok(state)
}
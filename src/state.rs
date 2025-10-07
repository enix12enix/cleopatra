// src/state.rs
// Application state module

use std::sync::Arc;
use sqlx::SqlitePool;
use crate::auth::{self, AuthProvider};
use crate::config::Config;
use crate::{config, database, suggestion};
use crate::background::writer::{DefaultWriter, Writer, WriterManager, WriterName};


#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub pool: SqlitePool,
    pub writer_manager: Arc<WriterManager>,
    pub auth_provider: Option<Arc<AuthProvider>>,
    pub execution_prefix_trie: Option<Arc<parking_lot::RwLock<suggestion::ExecutionPrefixTrie>>>,
}

impl AppState {
    #[allow(dead_code)]
    pub fn shared(self) -> Arc<Self> {
        Arc::new(self)
    }
}

/// Initialize the application state
pub async fn init_state() -> Result<AppState, Box<dyn std::error::Error>> {
    // Load configuration from environment / file
    let config = config::Config::from_env()?;
    let config = Arc::new(config); // wrap config in Arc

    // Initialize SQLite connection pools
    let (main_pool, writer_pool) = database::init_db(&config).await?;

    // Initialize writer manager
    let mut writer_manager = WriterManager::new();
    let default_writer = DefaultWriter::new(&config, writer_pool).await;
    writer_manager.insert(WriterName::Main, default_writer);

    // Initialize auth provider if enabled
    let auth_provider = if config.auth.enabled {
        Some(Arc::new(auth::AuthProvider::new(&config)?))
    } else {
        None
    };

    // Conditionally initialize execution prefix trie based on configuration
    let execution_prefix_trie = if config.execution_suggest.enabled {
        let trie = suggestion::ExecutionPrefixTrie::build_from_executions(
            &main_pool,
            config.execution_suggest.min_query_len,
            config.execution_suggest.max_query_len,
            config.execution_suggest.max_candidates,
        ).await?;
        Some(Arc::new(parking_lot::RwLock::new(trie)))
    } else {
        None
    };

    Ok(AppState { 
        config,
        pool: main_pool,
        writer_manager: Arc::new(writer_manager),
        auth_provider,
        execution_prefix_trie,
    })
}

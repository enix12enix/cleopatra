use std::{sync::Arc, time::Instant};
use anyhow::Result;
use tokio::task::JoinHandle;

use crate::{background::scheduler::{new_task, ScheduledTask}, config::DataRetentionConfig, database, state::AppState};

#[async_trait::async_trait]
pub trait Datasource: Send + Sync {
    async fn clean_up(&self, days: u32) -> Result<()>;
}

pub async fn clean_up_data<T>(ds: &T, config: &DataRetentionConfig)
where
    T: Datasource,
{
    println!("Clean up data started...");
    let start = Instant::now();

    if let Err(e) = T::clean_up(ds, config.period_in_day).await {
        eprintln!("Cleanup failed: {:?}", e);
    }

    let elapsed = start.elapsed();

    println!("Clean up data done in {:.2?}", elapsed);
}

/// Implement DatabasePool for SqlitePool to be used in scheduler
#[async_trait::async_trait]
impl Datasource for sqlx::SqlitePool {
    async fn clean_up(&self, days: u32) -> anyhow::Result<()> {
        database::clean_up_db(self, days).await
    }
}

// define tasks
pub fn sqlite_clean_up_task(state: Arc<AppState>) -> Option<ScheduledTask<AppState>> {
    let cfg = state.config.data_retention.get("main")?;
    if !cfg.enabled {
        return None;
    }

    let cfg_cloned = cfg.clone();
    Some(new_task(cfg_cloned.cron.clone(), move |state: Arc<AppState>| {
        let pool = state.pool.clone();
        let data_retention_cfg = cfg_cloned.clone();

        tokio::spawn(async move {
            clean_up_data(&pool, &data_retention_cfg).await;
        }) as JoinHandle<()>
    }))
}
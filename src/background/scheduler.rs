use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};
use anyhow::Result;

use crate::{background::tasks::create_tasks, state::AppState};

pub struct ScheduledTask<T>
where
    T: Send + Sync + 'static,
{
    pub cron: String,
    pub task: Arc<dyn Fn(Arc<T>) -> tokio::task::JoinHandle<()> + Send + Sync>,
}


pub fn new_task<T, F>(cron: String, func: F) -> ScheduledTask<T>
where
    T: Send + Sync + 'static,
    F: Fn(Arc<T>) -> tokio::task::JoinHandle<()> + Send + Sync + 'static,
{
    ScheduledTask {
        cron,
        task: Arc::new(func),
    }
}

/// Create a scheduler from a list of tasks
async fn create_scheduler<T>(
    shared: Arc<T>,
    tasks: Vec<ScheduledTask<T>>,
) ->  Result<JobScheduler>
where
    T: Send + Sync + 'static,
{
    let sched = JobScheduler::new().await?;

    for scheduled in tasks {
        let shared_clone = Arc::clone(&shared);
        let cron_expr = scheduled.cron.clone();

        println!("Scheduling task with cron: {}", cron_expr);

        sched
            .add(
                Job::new_async(cron_expr.as_str(), move |_uuid, _l| {
                    let shared = Arc::clone(&shared_clone);
                    let task = Arc::clone(&scheduled.task);
                    Box::pin(async move {
                        println!("Running task scheduled");
                        if let Err(e) = task(shared).await {
                            eprintln!("Task failed: {:?}", e);
                        }
                    })
                })?,
            )
            .await?;
    }

    Ok(sched)
}

pub async fn start_scheduler(state: Arc<AppState>) -> Result<JobScheduler> {
    let tasks = create_tasks(Arc::clone(&state));
    let scheduler = super::scheduler::create_scheduler(Arc::clone(&state), tasks).await?;
    scheduler.start().await?;
    Ok(scheduler)
}
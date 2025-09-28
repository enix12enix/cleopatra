
use std::sync::Arc;
use crate::{background::{scheduler::ScheduledTask, tasks::sweeper::sqlite_clean_up_task}, state::AppState};

pub mod sweeper;


/// Build scheduled tasks using the entire application state
pub fn create_tasks(state: Arc<AppState>) -> Vec<ScheduledTask<AppState>> {
    let mut tasks = Vec::new();

     if let Some(task) = sqlite_clean_up_task(Arc::clone(&state)) {
        tasks.push(task);
    }

    // add more tasks here.

    tasks
}
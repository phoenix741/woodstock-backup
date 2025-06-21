use std::future::Future;

use log::debug;
use tokio::{task::JoinHandle, task_local};

// Task-local variable to store the current context ID
task_local! {
  pub static CURRENT_CONTEXT_ID: usize;
}

pub fn get_current_context_id() -> usize {
    // Retrieve the current context ID, defaulting to 0 if not set
    CURRENT_CONTEXT_ID.try_with(|id| *id).unwrap_or(0)
}

// Helper function to spawn tasks with context
pub fn spawn_with_context<F, R>(future: F) -> JoinHandle<R>
where
    F: Future<Output = R> + Send + 'static,
    R: Send + 'static,
{
    // Capture current context
    let context_id = CURRENT_CONTEXT_ID.try_with(|id| *id).unwrap_or(0);

    debug!("Spawning task with context ID: {}", context_id);

    // Spawn with context propagation
    tokio::spawn(async move { CURRENT_CONTEXT_ID.scope(context_id, future).await })
}

pub fn spawn_with_context_id<F, R>(context_id: usize, future: F) -> JoinHandle<R>
where
    F: Future<Output = R> + Send + 'static,
    R: Send + 'static,
{
    debug!("Spawning task with context ID (forced): {}", context_id);

    // Spawn with context propagation
    tokio::spawn(async move { CURRENT_CONTEXT_ID.scope(context_id, future).await })
}

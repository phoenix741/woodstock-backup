//! Application state for the job worker binary.

use crate::{
    jobs::{progress::ProgressPublisher, storage::ApalisRedisStorage},
    shared_state::SharedState,
};
use eyre::{Result, WrapErr};
use std::sync::Arc;
use woodstock::config::Configuration;

/// Application state for the Apalis job worker.
///
/// Embeds a [`SharedState`] (infrastructure shared with the API server) plus
/// worker-specific resources. Implements [`std::ops::Deref`] so shared fields
/// are directly accessible on the state object.
#[derive(Clone)]
pub struct ApiWorkerState {
    /// Shared infrastructure (config, scheduler, hosts, backups, resolver, job_utility).
    pub shared: SharedState,

    /// Apalis Redis-based job storage.
    pub apalis_redis_storage: Arc<ApalisRedisStorage>,

    /// Publisher used to push job-progress updates to Redis Pub/Sub.
    pub progress_publisher: ProgressPublisher,
}

impl std::ops::Deref for ApiWorkerState {
    type Target = SharedState;

    fn deref(&self) -> &Self::Target {
        &self.shared
    }
}

impl ApiWorkerState {
    /// Create new job worker state.
    ///
    /// # Errors
    ///
    /// Returns an error if Redis is unreachable or any sub-component fails.
    pub async fn new(config: Arc<Configuration>) -> Result<Self> {
        let redis_url = config.redis_url();
        let (shared, redis_client) = SharedState::new(config.clone()).await?;

        let apalis_redis_storage = Arc::new(ApalisRedisStorage::new(&redis_url).await?);
        let progress_publisher = ProgressPublisher::new(redis_client)
            .await
            .wrap_err("Failed to create ProgressPublisher — is Redis reachable?")?;

        Ok(Self {
            shared,
            apalis_redis_storage,
            progress_publisher,
        })
    }
}

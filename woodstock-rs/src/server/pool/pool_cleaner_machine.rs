use std::{path::PathBuf, sync::Arc};

use eyre::Result;
use tokio::sync::{mpsc, Mutex};
use tracing::{error, warn, Instrument};

use crate::{
    config::Configuration,
    pool::PoolManager,
    utils::lock_redis::{LockOperation, PoolLockOperation, PoolLockRedis},
    EventPoolCleanedInformation, EventSource,
};

use super::{
    pool_cleaner::PoolCleaner, pool_cleaner::PoolProgression, pool_cleaner_state::CleanerState,
};

pub struct PoolCleanerMachine {
    /// The configuration for the pool cleaner.
    config: Arc<Configuration>,
    /// The `PoolCleaner` instance responsible for performing cleaning operations on the storage pool.
    cleaner: PoolCleaner,
    /// Optional target path specifying where the cleaning operation should be applied.
    target: Option<PathBuf>,
    /// The event source used to provide context or events for the cleaning process.
    source: EventSource,
    /// Shared state of the cleaning process, protected by a mutex for safe concurrent access.
    state: Arc<Mutex<CleanerState>>,
    /// Optional channel for sending state updates to observers or other components.
    state_tx: Option<mpsc::Sender<CleanerState>>,
}

impl PoolCleanerMachine {
    /// Creates a new instance of `PoolCleanerMachine`.
    ///
    /// # Arguments
    /// * `config` - The configuration for the pool cleaner.
    /// * `target` - An optional target path for cleaning.
    /// * `source` - The event source.
    /// * `state_tx` - An optional channel for sending state updates.
    ///
    /// # Returns
    ///
    /// A new instance of `PoolCleanerMachine`.
    #[must_use]
    pub fn new(
        target: Option<PathBuf>,
        source: EventSource,
        state_tx: Option<mpsc::Sender<CleanerState>>,
        config: Arc<Configuration>,
    ) -> Self {
        let cleaner = PoolCleaner::new(config.clone());
        let state = CleanerState::new();

        Self {
            config,
            cleaner,
            target,
            source,
            state: Arc::new(Mutex::new(state)),
            state_tx,
        }
    }

    /// Sends the current state to the state channel.
    ///
    /// # Errors
    ///
    /// Logs an error if sending the state update fails.
    async fn send_state(&self) {
        if let Some(state_tx) = &self.state_tx {
            let state = self.state.lock().await;

            if let Err(e) = state_tx.send(state.clone()).await {
                error!("Failed to send state update: {}", e);
            }
        }
    }

    /// Applies pending Pool V3 publication/removal integrations before compaction.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operations are applied successfully.
    /// * `Err(eyre::Report)` if an error occurs during the operation.
    ///
    /// # Errors
    ///
    /// Returns an error if pending Pool V3 integrations cannot be applied.
    async fn apply_pending_operations(&self) -> Result<()> {
        PoolManager::new(self.config.clone())
            .apply_pending_operations()
            .await
    }

    /// Initializes the cleaning process.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the initialization succeeds.
    /// * `Err(eyre::Report)` if an error occurs during initialization.
    ///
    /// # Errors
    ///
    /// Returns an error if the initialization fails.
    async fn init_cleaning(&self) -> Result<()> {
        {
            let mut state = self.state.lock().await;
            state.start_initialization();
        }
        self.send_state().await;

        let result = self.cleaner.clean_unused_max().await;

        {
            let mut state = self.state.lock().await;
            state.process_initialization_result(result)?;
        }
        self.send_state().await;
        Ok(())
    }

    /// Executes the cleaning process.
    ///
    /// # Returns
    ///
    /// * `Ok(EventPoolCleanedInformation)` if the cleaning succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the cleaning process.
    ///
    /// # Errors
    ///
    /// Returns an error if the cleaning fails.
    async fn execute_cleaning(&self) -> Result<EventPoolCleanedInformation> {
        {
            let mut state = self.state.lock().await;
            state.start_cleaning();
        }
        self.send_state().await;

        // Create a channel to receive progress updates
        let (progress_tx, mut progress_rx) = mpsc::channel::<PoolProgression>(10);

        // Create a task to process progress updates
        let state_clone = self.state.clone();
        let state_tx_clone = self.state_tx.clone();

        let progress_task = tokio::spawn(
            async move {
                while let Some(progress) = progress_rx.recv().await {
                    let mut state = state_clone.lock().await;
                    state.process_cleaning_progress(
                        progress.progress_current,
                        progress.file_size,
                        progress.compressed_file_size,
                    );

                    // Send the updated state
                    if let Some(tx) = &state_tx_clone {
                        if let Err(e) = tx.send(state.clone()).await {
                            error!("Failed to send state update during cleaning: {}", e);
                        }
                    }
                }
            }
            .in_current_span(),
        );

        // Execute the cleaning
        let result = self
            .cleaner
            .clean_unused_pool(self.target.clone(), self.source, Some(progress_tx))
            .await;

        // Wait for the progress task to complete
        if let Err(e) = progress_task.await {
            error!("Error in cleaning progression task: {}", e);
        }

        let result = {
            let mut state = self.state.lock().await;
            state.process_cleaning_result(result)
        };
        self.send_state().await;

        result
    }

    /// Inner cleaning work, runs after the exclusive lock is acquired.
    ///
    /// Separated from `execute()` so the latter can wrap this in a `tokio::select!`
    /// against the lock's cancellation token.
    async fn run_cleaner_core(&self) -> Result<CleanerState> {
        // Apply pending Pool V3 integrations (protected by EXCLUSIVE lock)
        self.apply_pending_operations().await?;

        // Initialize
        self.init_cleaning().await?;

        // Execute the cleaning
        let _ = self.execute_cleaning().await?;

        let state = {
            let state = self.state.lock().await;
            state.clone()
        };

        Ok(state)
    }

    /// Executes the entire cleaning process, including initialization and cleaning.
    ///
    /// Acquires an exclusive Redis lock on the pool, then runs the cleaner via
    /// `run_cleaner_core()` wrapped in a `tokio::select!` against the lock's
    /// cancellation token. If the lock heartbeat stops (e.g. the host goes to
    /// sleep and the Redis TTL expires), the token fires and the cleaner aborts
    /// immediately with an error rather than running concurrently with a
    /// re-enqueued duplicate job.
    ///
    /// # Returns
    ///
    /// * `Ok(CleanerState)` if the process succeeds.
    /// * `Err(eyre::Report)` if an error occurs or the lock is lost mid-run.
    ///
    /// # Errors
    ///
    /// Returns an error if any step of the process fails, or if the
    /// exclusive Redis lock is lost during execution.
    pub async fn execute(&self) -> Result<CleanerState> {
        // SAFETY CHECK: Verify pool is not dirty before proceeding
        PoolManager::new(self.config.clone())
            .assert_clean("pool cleanup")
            .await?;

        // Acquire EXCLUSIVE lock BEFORE applying refcnt operations.
        // This prevents race conditions with concurrent backups.
        let redis_url = self.config.redis_url();
        let lock = PoolLockRedis::new_with_path(
            &redis_url,
            &self.config.path.pool_path,
            LockOperation::Pool(PoolLockOperation::ExecuteCleaning),
        )
        .await?
        .lock_exclusive()
        .await?;

        let cancel_token = lock.cancellation_token().clone();

        tokio::select! {
            biased;
            res = self.run_cleaner_core() => res,
            _ = cancel_token.cancelled() => {
                warn!(
                    "Cleanup aborted: Redis lock was lost for pool {:?} (host likely suspended during sleep)",
                    self.config.path.pool_path
                );
                Err(eyre::eyre!(
                    "Cleanup aborted: Redis lock lost (process was likely suspended during sleep)"
                ))
            }
        }
    }

    /// Retrieves the current state of the pool cleaner.
    ///
    /// # Returns
    ///
    /// The current `CleanerState`.
    pub async fn get_state(&self) -> CleanerState {
        self.state.lock().await.clone()
    }
}

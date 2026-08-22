use std::{collections::HashSet, sync::Arc};

use chrono::Local;
use eyre::Result;
use tokio::sync::{mpsc, Mutex};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn, Instrument};

use crate::{
    config::{Backups, Configuration, Hosts, DEFAULT_CHANNEL_BUFFER_SIZE},
    pool::{CheckUnusedOutcome, FsckMissingCount, FsckUnusedCount, PoolManager, Refcnt},
    utils::lock_redis::{LockOperation, PoolLockOperation, PoolLockRedis},
    EventPoolInformation, EventSource,
};

use super::{fsck::FsckProgression, fsck::PoolFsck, fsck_state::FsckState};

pub struct FsckMachine {
    /// The configuration for the fsck process.
    config: Arc<Configuration>,
    /// The `PoolFsck` instance responsible for performing the fsck operations on the storage pool.
    fsck: PoolFsck,
    /// The event source used to provide context or events for the fsck process.
    source: EventSource,
    /// Indicates whether the fsck process should be executed in dry-run mode (no changes applied).
    dry_run: bool,
    /// Indicates whether chunk verification should be performed during the fsck process.
    verify_chunks: bool,
    /// Indicates whether reference count and unused space verification should be skipped.
    skip_ref_unused: bool,
    /// Shared state of the fsck process, protected by a mutex for safe concurrent access.
    state: Arc<Mutex<FsckState>>,
    /// Optional channel for sending state updates to observers or other components.
    state_tx: Option<mpsc::Sender<FsckState>>,
    /// Cancelled when the user requests to stop this fsck run. Distinct from
    /// the pool lock's own cancellation token consumed by `execute()`'s
    /// `select!` (which fires on lock loss and aborts with an error, bypassing
    /// finalization) — this one is threaded into the actual verification
    /// loops so a user cancel flows through normal completion instead.
    cancel_token: CancellationToken,
}

impl FsckMachine {
    /// Creates a new instance of `FsckMachine`.
    ///
    /// # Arguments
    /// * `config` - The configuration for the fsck process.
    /// * `source` - The event source for the fsck process.
    /// * `dry_run` - Whether to perform a dry run.
    /// * `verify_chunks` - Whether to verify chunks.
    /// * `skip_ref_unused` - Whether to skip reference count and unused space verification.
    /// * `state_tx` - An optional channel for sending state updates.
    ///
    /// # Returns
    ///
    /// A new instance of `FsckMachine`.
    #[must_use]
    pub fn new(
        source: EventSource,
        dry_run: bool,
        verify_chunks: bool,
        skip_ref_unused: bool,
        state_tx: Option<mpsc::Sender<FsckState>>,
        config: Arc<Configuration>,
        hosts: Arc<Hosts>,
        backups: Arc<Backups>,
        cancel_token: CancellationToken,
    ) -> Self {
        let fsck = PoolFsck::new(config.clone(), hosts, backups);
        let state = FsckState::new(dry_run);

        Self {
            config,
            fsck,
            source,
            dry_run,
            verify_chunks,
            skip_ref_unused,
            state: Arc::new(Mutex::new(state)),
            state_tx,
            cancel_token,
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

    /// If the user has requested cancellation, marks the shared state
    /// `Cancelled` and returns `true` — callers use this between phases to
    /// stop before starting the next one.
    async fn check_cancelled(&self) -> bool {
        if self.cancel_token.is_cancelled() {
            let mut state = self.state.lock().await;
            state.cancel();
            true
        } else {
            false
        }
    }

    /// Applies pending refcnt operations.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operations are applied successfully.
    /// * `Err(())` if the operations fail (non-fatal, will be handled by check_pool_integrity).
    async fn apply_refcnt_operations(&self) -> Result<(), ()> {
        {
            let mut state = self.state.lock().await;
            state.start_applying_refcnt();
        }
        self.send_state().await;

        let current_time = Local::now();
        let result = PoolManager::new(self.config.clone())
            .apply_pending(&current_time)
            .await;

        {
            let mut state = self.state.lock().await;
            match state.process_applying_refcnt_result(result) {
                Ok(_) => (),
                Err(e) => {
                    debug!("apply_refcnt_operations failed (will be fixed by check_pool_integrity): {}", e);
                    self.send_state().await;
                    return Err(());
                }
            }
        }
        self.send_state().await;
        Ok(())
    }

    /// Initializes the fsck process.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the initialization succeeds.
    /// * `Err(eyre::Report)` if an error occurs during initialization.
    ///
    /// # Errors
    ///
    /// Returns an error if any step of the initialization process fails.
    async fn initialize(&self) -> Result<()> {
        {
            let mut state = self.state.lock().await;
            state.start_initialization();
        }
        self.send_state().await;

        // Get initialization results for each verification type
        let refcnt_max_result = self.fsck.verify_refcnt_max().await;
        let unused_max_result = self.fsck.verify_unused_max().await;
        let missing_max_result = self.fsck.verify_missing_max().await;
        let chunk_max_result = if self.verify_chunks {
            self.fsck.verify_chunk_max().await
        } else {
            Ok(Vec::new())
        };

        {
            let mut state = self.state.lock().await;
            return state.process_initialization_result(
                refcnt_max_result,
                unused_max_result,
                missing_max_result,
                chunk_max_result,
            );
        }
    }

    /// Verifies the reference count.
    ///
    /// # Arguments
    ///
    /// * `dry_run` - Whether to perform a dry run.
    ///
    /// # Returns
    ///
    /// * `Ok(EventRefCountInformation)` if the verification succeeds.
    /// * `Err(eyre::Report)` if an error occurs during verification.
    ///
    /// # Errors
    ///
    /// Returns an error if any step of the reference count verification process fails.
    async fn verify_refcnt(&self, dry_run: bool) -> Result<FsckProgression> {
        {
            let mut state = self.state.lock().await;
            state.start_verify_refcnt();
        }
        self.send_state().await;

        // Create a channel to receive progression updates
        let (progress_tx, mut progress_rx) =
            mpsc::channel::<FsckProgression>(DEFAULT_CHANNEL_BUFFER_SIZE);

        // Create a task to process progression updates
        let state_clone = self.state.clone();
        let state_tx_clone = self.state_tx.clone();

        let progress_task = tokio::spawn(
            async move {
                while let Some(progress) = progress_rx.recv().await {
                    let mut state = state_clone.lock().await;
                    state.process_verify_refcnt_progress(&progress);

                    // Send updated state
                    if let Some(tx) = &state_tx_clone {
                        if let Err(e) = tx.send(state.clone()).await {
                            error!(
                                "Failed to send state update during refcnt verification: {}",
                                e
                            );
                        }
                    }
                }
            }
            .in_current_span(),
        );

        // Execute verification
        let result = self
            .fsck
            .verify_refcnt(dry_run, Some(progress_tx), &self.cancel_token)
            .await;

        // Wait for progression task to complete
        if let Err(e) = progress_task.await {
            error!("Error in refcnt verification progression task: {}", e);
        }

        let result = {
            let mut state = self.state.lock().await;
            state.process_verify_refcnt_result(result)
        };
        self.send_state().await;
        result
    }

    /// Verifies the unused space.
    ///
    /// Only walks the pool directory and classifies chunks against the loaded refcnt — it
    /// does not check for refcnt entries missing from disk; that's [`Self::verify_missing`],
    /// which needs the `refcnt`/`seen` returned here (see [`CheckUnusedOutcome`]).
    ///
    /// # Arguments
    ///
    /// * `dry_run` - Whether to perform a dry run.
    ///
    /// # Returns
    ///
    /// * `Ok(CheckUnusedOutcome)` if the verification succeeds.
    /// * `Err(eyre::Report)` if an error occurs during verification.
    ///
    /// # Errors
    ///
    /// Returns an error if any step of the unused space verification process fails.
    async fn verify_unused(&self, dry_run: bool) -> Result<CheckUnusedOutcome> {
        {
            let mut state = self.state.lock().await;
            state.start_verify_unused();
        }
        self.send_state().await;

        // Create a channel to receive progression updates
        let (progress_tx, mut progress_rx) =
            mpsc::channel::<FsckUnusedCount>(DEFAULT_CHANNEL_BUFFER_SIZE);

        // Create a task to process progression updates
        let state_clone = self.state.clone();
        let state_tx_clone = self.state_tx.clone();

        let progress_task = tokio::spawn(
            async move {
                while let Some(progress) = progress_rx.recv().await {
                    let mut state = state_clone.lock().await;
                    state.process_verify_unused_progress(&progress);

                    // Send updated state
                    if let Some(tx) = &state_tx_clone {
                        if let Err(e) = tx.send(state.clone()).await {
                            error!(
                                "Failed to send state update during unused verification: {}",
                                e
                            );
                        }
                    }
                }
            }
            .in_current_span(),
        );

        // Execute verification
        let result = self
            .fsck
            .verify_unused(dry_run, Some(progress_tx), &self.cancel_token)
            .await;

        // Wait for progression task to complete
        if let Err(e) = progress_task.await {
            error!("Error in unused verification progression task: {}", e);
        }

        // Split off `refcnt`/`seen` before handing the count through the usual
        // process_verify_*_result state-update path — they're plain data `verify_missing`
        // needs next, not part of the progression state.
        let (refcnt_seen, count_result) = match result {
            Ok(outcome) => (Some((outcome.refcnt, outcome.seen)), Ok(outcome.count)),
            Err(e) => (None, Err(e)),
        };
        let count_result = {
            let mut state = self.state.lock().await;
            state.process_verify_unused_result(count_result)
        };
        // Send state after releasing the lock
        self.send_state().await;

        match (count_result, refcnt_seen) {
            (Ok(count), Some((refcnt, seen))) => Ok(CheckUnusedOutcome {
                count,
                refcnt,
                seen,
            }),
            (Err(e), _) => Err(e),
            (Ok(_), None) => unreachable!("refcnt_seen is Some exactly when result was Ok"),
        }
    }

    /// Verifies refcnt entries that reference a chunk absent from disk.
    ///
    /// # Arguments
    ///
    /// * `refcnt` - The `Refcnt` loaded by the preceding [`Self::verify_unused`] call.
    /// * `seen` - The set of chunk hashes [`Self::verify_unused`] found present on disk.
    ///
    /// # Returns
    ///
    /// * `Ok(FsckMissingCount)` if the verification succeeds.
    /// * `Err(eyre::Report)` if an error occurs during verification.
    ///
    /// # Errors
    ///
    /// Returns an error if any step of the missing chunks verification process fails.
    async fn verify_missing(
        &self,
        refcnt: &Refcnt,
        seen: &HashSet<[u8; 32]>,
    ) -> Result<FsckMissingCount> {
        {
            let mut state = self.state.lock().await;
            state.start_verify_missing();
        }
        self.send_state().await;

        // Create a channel to receive progression updates
        let (progress_tx, mut progress_rx) =
            mpsc::channel::<FsckMissingCount>(DEFAULT_CHANNEL_BUFFER_SIZE);

        // Create a task to process progression updates
        let state_clone = self.state.clone();
        let state_tx_clone = self.state_tx.clone();

        let progress_task = tokio::spawn(
            async move {
                while let Some(progress) = progress_rx.recv().await {
                    let mut state = state_clone.lock().await;
                    state.process_verify_missing_progress(&progress);

                    // Send updated state
                    if let Some(tx) = &state_tx_clone {
                        if let Err(e) = tx.send(state.clone()).await {
                            error!(
                                "Failed to send state update during missing chunks verification: {}",
                                e
                            );
                        }
                    }
                }
            }
            .in_current_span(),
        );

        // Execute verification
        let result = self
            .fsck
            .verify_missing(refcnt, seen, Some(progress_tx), &self.cancel_token)
            .await;

        // Wait for progression task to complete
        if let Err(e) = progress_task.await {
            error!("Error in missing chunks verification progression task: {}", e);
        }

        // Process result and update state, sans let inutile ni move
        let result = {
            let mut state = self.state.lock().await;
            state.process_verify_missing_result(result)
        };
        // Send state after releasing the lock
        self.send_state().await;
        result
    }

    /// Verifies the chunks.
    ///
    /// # Returns
    ///
    /// * `Ok(EventRefCountInformation)` if the verification succeeds.
    /// * `Err(eyre::Report)` if an error occurs during verification.
    ///
    /// # Errors
    ///
    /// Returns an error if any step of the chunk verification process fails.
    async fn verify_chunk(&self) -> Result<FsckProgression> {
        {
            let mut state = self.state.lock().await;
            state.start_verify_chunk();
        }
        self.send_state().await;

        // Create a channel to receive progression updates
        let (progress_tx, mut progress_rx) =
            mpsc::channel::<FsckProgression>(DEFAULT_CHANNEL_BUFFER_SIZE);

        // Create a task to process progression updates
        let state_clone = self.state.clone();
        let state_tx_clone = self.state_tx.clone();

        let progress_task = tokio::spawn(
            async move {
                while let Some(progress) = progress_rx.recv().await {
                    let mut state = state_clone.lock().await;
                    state.process_verify_chunk_progress(&progress);

                    // Send updated state
                    if let Some(tx) = &state_tx_clone {
                        if let Err(e) = tx.send(state.clone()).await {
                            error!(
                                "Failed to send state update during chunk verification: {}",
                                e
                            );
                        }
                    }
                }
            }
            .in_current_span(),
        );

        // Execute verification
        let result = self
            .fsck
            .verify_chunk(Some(progress_tx), &self.cancel_token)
            .await;

        // Wait for progression task to complete
        if let Err(e) = progress_task.await {
            error!("Error in chunk verification progression task: {}", e);
        }

        // Process result and update state, sans let inutile ni move
        let result = {
            let mut state = self.state.lock().await;
            state.process_verify_chunk_result(result)
        };
        // Send state after releasing the lock
        self.send_state().await;
        result
    }

    /// Inner fsck work, runs after the exclusive lock is acquired.
    ///
    /// Separated from `execute()` so the latter can wrap this in a `tokio::select!`
    /// against a cancellation token, aborting cleanly if the lock is lost mid-run.
    async fn run_fsck_core(&self) -> Result<FsckState> {
        // Try to apply pending refcnt operations (may fail if pool is dirty - not fatal)
        if let Err(_) = self.apply_refcnt_operations().await {
            debug!("Pending refcnt operations not applied (pool may be dirty - will be cleaned by check_pool_integrity)");
        }

        // Checked before the start event so a cancel requested while still
        // queued (or right as the run begins) never even logs a start/end
        // pair for a run that did nothing.
        if self.check_cancelled().await {
            self.send_state().await;
            let state = self.state.lock().await;
            return Ok(state.clone());
        }

        // Create start event
        let id = self.fsck.create_event_start(self.source).await?;

        info!(
            "Initializing fsck process for pool: {:?}",
            self.config.path.pool_path
        );
        // Initialization
        self.initialize().await?;

        let mut information = EventPoolInformation::default();
        information.fix = !self.dry_run;

        // Checked between every phase below (not just once) — the point is
        // to stop before starting the *next* phase, not to interrupt one
        // already running (each `verify_*` call already checks the token
        // internally, in its own loop, for that).
        let mut cancelled = self.check_cancelled().await;

        if !cancelled && !self.skip_ref_unused {
            debug!("Verifying reference counts...");

            // Reference count verification
            let refcnt_result = self.verify_refcnt(self.dry_run).await;
            match refcnt_result {
                Ok(refcnt_info) => {
                    information.refcount = refcnt_info.total_count as u64;
                    information.refcount_error = refcnt_info.error_count as u64;
                }
                Err(e) => {
                    error!("Error during refcnt verification: {}", e);
                    {
                        let mut state = self.state.lock().await;
                        state.fail();
                    }
                    self.send_state().await;
                    return Err(e);
                }
            }

            cancelled = self.check_cancelled().await;
            if !cancelled {
                debug!("Verifying unused files...");

                let unused_result = self.verify_unused(self.dry_run).await;
                let outcome = match unused_result {
                    Ok(outcome) => {
                        information.in_refcnt = outcome.count.in_refcnt as u64;
                        information.in_unused = outcome.count.in_unused as u64;
                        information.in_nothing = outcome.count.in_nothing as u64;
                        outcome
                    }
                    Err(e) => {
                        error!("Error during unused verification: {}", e);
                        {
                            let mut state = self.state.lock().await;
                            state.fail();
                        }
                        self.send_state().await;
                        return Err(e);
                    }
                };

                // Checked before starting the missing-chunk scan — a cancel that arrived
                // during (or right after) the walk above must skip this phase entirely
                // rather than pay its cost and persist a `missing` snapshot built from a
                // `seen` set that a partial walk can't be trusted to represent.
                cancelled = self.check_cancelled().await;
                if !cancelled {
                    debug!("Verifying missing chunks...");

                    let missing_result = self.verify_missing(&outcome.refcnt, &outcome.seen).await;
                    match missing_result {
                        Ok(missing_info) => {
                            information.missing = missing_info.missing as u64;
                        }
                        Err(e) => {
                            error!("Error during missing chunks verification: {}", e);
                            {
                                let mut state = self.state.lock().await;
                                state.fail();
                            }
                            self.send_state().await;
                            return Err(e);
                        }
                    }
                    cancelled = self.check_cancelled().await;
                }
            }
        }

        if !cancelled && self.verify_chunks {
            debug!("Verifying chunks...");

            let chunk_result = self.verify_chunk().await;
            match chunk_result {
                Ok(chunk_info) => {
                    information.chunk_count = chunk_info.total_count as u64;
                    information.chunk_error = chunk_info.error_count as u64;
                }
                Err(e) => {
                    error!("Error during chunk verification: {}", e);
                    {
                        let mut state = self.state.lock().await;
                        state.fail();
                    }
                    self.send_state().await;
                    return Err(e);
                }
            }
        }

        // Re-checked here (not just relying on the `cancelled` flag from the
        // phase boundaries above) because `verify_chunk`'s own internal loop
        // — the last phase — can itself have broken early on a cancel that
        // arrived mid-verification.
        cancelled = cancelled || self.cancel_token.is_cancelled();

        debug!("Registering fsck process completion event...");
        {
            let mut state = self.state.lock().await;
            if cancelled {
                state.cancel();
            } else {
                state.complete();
            }
        }
        self.send_state().await;

        self.fsck
            .create_event_refcnt_end(&id, information, self.source)
            .await?;

        debug!("End of fsck process ...");

        let state = {
            let state = self.state.lock().await;
            state.clone()
        };

        Ok(state)
    }

    /// Executes the fsck process.
    ///
    /// Acquires an exclusive Redis lock on the pool, then runs the fsck via
    /// `run_fsck_core()` wrapped in a `tokio::select!` against the lock's
    /// cancellation token.  If the lock heartbeat stops (e.g. the host goes to
    /// sleep and the Redis TTL expires), the token fires and the fsck aborts
    /// immediately with an error rather than running concurrently with a
    /// re-enqueued duplicate job.
    ///
    /// # Returns
    ///
    /// * `Ok(FsckState)` if the fsck process succeeds.
    /// * `Err(eyre::Report)` if an error occurs or the lock is lost mid-run.
    ///
    /// # Errors
    ///
    /// Returns an error if any step of the fsck process fails, or if the
    /// exclusive Redis lock is lost during execution.
    pub async fn execute(&self) -> Result<FsckState> {
        info!(
            "Starting fsck process for pool: {:?}",
            self.config.path.pool_path
        );

        // DIRTY STATE CHECK: Warn in dry-run mode only
        if self.dry_run {
            if let Some(dirty_file) = PoolManager::new(self.config.clone()).is_dirty().await? {
                warn!(
                    "Pool is DIRTY (file: {:?}) - a previous refcnt operation crashed. Run without --dry-run to clean up.",
                    dirty_file
                );
            }
        }

        // Acquire EXCLUSIVE lock BEFORE any operations.
        // This prevents race conditions with concurrent backups.
        info!(
            "Acquiring exclusive lock for pool: {:?}",
            self.config.path.pool_path
        );
        let redis_url = self.config.redis_url();
        let lock = PoolLockRedis::new_with_path(
            &redis_url,
            &self.config.path.pool_path,
            LockOperation::Pool(PoolLockOperation::Fsck),
        )
        .await?
        .lock_exclusive()
        .await?;

        // Clone the cancellation token BEFORE passing the lock into the select.
        // The token fires if the heartbeat detects that the Redis key expired
        // (e.g. the host slept for > LOCK_TTL seconds without a heartbeat tick).
        let cancel_token = lock.cancellation_token().clone();

        // Lock is held for the entire duration of the select!.
        // It is dropped (and released in Redis) when execute() returns,
        // regardless of which branch wins.
        tokio::select! {
            biased; // prefer work completion if both fire simultaneously
            res = self.run_fsck_core() => res,
            _ = cancel_token.cancelled() => {
                warn!(
                    "Fsck aborted: Redis lock was lost for pool {:?} (host likely suspended during sleep)",
                    self.config.path.pool_path
                );
                {
                    let mut state = self.state.lock().await;
                    state.complete();
                }
                self.send_state().await;
                Err(eyre::eyre!(
                    "Fsck aborted: Redis lock lost (process was likely suspended during sleep)"
                ))
            }
        }
    }

    /// Gets the current state of the fsck process.
    ///
    /// # Returns
    ///
    /// The current state of the fsck process.
    pub async fn get_state(&self) -> FsckState {
        self.state.lock().await.clone()
    }
}

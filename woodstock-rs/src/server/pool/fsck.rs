use std::{collections::HashSet, sync::Arc, time::SystemTime};
use tokio::sync::mpsc;

use eyre::Result;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, Instrument};
use uuid::Uuid;

use crate::{
    config::{Backups, Configuration, Hosts, DEFAULT_CHANNEL_BUFFER_SIZE},
    events::append_events,
    pool::{
        check_backup_integrity, check_host_integrity, check_missing, check_pool_integrity,
        check_unused, CheckUnusedOutcome, FsckMissingCount, FsckUnusedCount, PoolChunkWrapper,
        Refcnt,
    },
    woodstock::event::Information,
    Event, EventPoolInformation, EventSource, EventStatus, EventStep, EventType,
};

/// # Pool Filesystem Check (fsck) Module
///
/// This module provides logic for verifying and repairing the integrity of the Woodstock backup pool.
/// It includes reference count checking, unused chunk detection, chunk integrity verification, and event logging.
///
/// ## Main Structures
///
/// - [`FsckProgression`]: Tracks progress and errors during fsck operations.
/// - [`PoolProgression`]: Tracks progress and statistics for pool operations.
/// - [`PoolFsck`]: Main struct for running fsck operations on the backup pool.
///
/// ## Main Methods
///
/// - [`PoolFsck::verify_refcnt_max`]: Calculates the maximum number of reference counts.
/// - [`PoolFsck::verify_refcnt`]: Verifies reference counts and logs events.
/// - [`PoolFsck::verify_unused_max`]: Calculates the maximum number of unused chunks.
/// - [`PoolFsck::verify_unused`]: Verifies unused chunks and logs events.
/// - [`PoolFsck::verify_chunk_max`]: Calculates the maximum number of chunks to check.
/// - [`PoolFsck::verify_chunk`]: Verifies chunk integrity and logs events.
///
/// ## Error Handling & Panics
///
/// - All public methods return `Result` and propagate errors using the `eyre` crate.
/// - Panics are not expected under normal operation; errors are returned as `Result`.
///
/// ## Thread Safety
///
/// This struct is not thread-safe by itself. Use with care in concurrent contexts.
///
/// ## See Also
///
/// - [`Refcnt`], [`PoolChunkWrapper`], [`Event`]: For related pool and event operations
#[derive(Clone, Debug)]
///
/// Tracks progress and errors during fsck operations.
///
/// This structure is used to monitor the progress and capture any errors that occur during the filesystem check (fsck) operations.
///
/// # Fields
///
/// * `error_count` - The number of errors encountered so far.
/// * `total_count` - The total number of items processed so far.
/// * `progress_current` - The current progress value (number of steps completed).
pub struct FsckProgression {
    pub error_count: usize,
    pub total_count: usize,
    pub progress_current: usize,
}

#[derive(Clone, Debug)]
/// Tracks progress and statistics for pool operations.
pub struct PoolProgression {
    pub progress_current: usize,

    pub file_count: usize,
    pub file_size: u64,
    pub compressed_file_size: u64,
}

#[derive(Clone)]
/// Main struct for running fsck operations on the backup pool.
pub struct PoolFsck {
    /// Configuration for the fsck operations.
    ///
    /// This struct is used to monitor the progress and capture any errors that occur during the filesystem check (fsck) operations.
    pub config: Arc<Configuration>,
    pub hosts_config: Arc<Hosts>,
    pub backups_config: Arc<Backups>,
}

impl PoolFsck {
    /// Creates a new [`PoolFsck`] from the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Reference to the Woodstock [`Configuration`] struct.
    ///
    /// # Returns
    ///
    /// A new instance of [`PoolFsck`].
    #[must_use]
    pub fn new(config: Arc<Configuration>, hosts: Arc<Hosts>, backups: Arc<Backups>) -> Self {
        PoolFsck {
            config,
            hosts_config: hosts,
            backups_config: backups,
        }
    }

    /// Creates a new event to mark the start of an operation.
    ///
    /// # Arguments
    /// * `event_type` - The type of the event.
    /// * `source` - The source of the event.
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` if the event was successfully created.
    /// * `Err(eyre::Report)` if an error occurred during the operation.
    ///
    /// # Errors
    /// This function returns an error if the event creation fails.
    pub async fn create_event_start(&self, source: EventSource) -> Result<Vec<u8>> {
        let id = Uuid::new_v4();
        let id = id.as_bytes();

        let event = Event {
            id: id.to_vec(),
            event_type: EventType::PoolChecked as i32,
            step: EventStep::Start as i32,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
            source: source as i32,
            user: String::new(),
            error_messages: Vec::new(),
            status: EventStatus::None as i32,

            information: None,
        };

        append_events(&self.config, &self.config.path.events_path, &[&event]).await?;

        Ok(id.to_vec())
    }

    /// Creates a new event to mark the end of a reference count operation.
    ///
    /// # Arguments
    /// * `id` - The identifier of the event.
    /// * `information` - Additional information about the reference count operation.
    ///
    /// # Returns
    /// * `Ok(())` if the event was successfully created.
    /// * `Err(eyre::Report)` if an error occurred during the operation.
    ///
    /// # Errors
    /// This function returns an error if the event creation fails.
    pub async fn create_event_refcnt_end(
        &self,
        id: &[u8],
        information: EventPoolInformation,
        source: EventSource,
    ) -> Result<()> {
        let event = Event {
            id: id.to_vec(),
            event_type: EventType::PoolChecked as i32,
            step: EventStep::End as i32,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
            source: source as i32,
            user: String::new(),
            error_messages: Vec::new(),
            status: if information.refcount_error > 0
                || information.chunk_error > 0
                || information.missing > 0
                || information.in_nothing > 0
            {
                EventStatus::GenericError
            } else {
                EventStatus::Success
            } as i32,

            information: Some(Information::Pool(information)),
        };

        append_events(&self.config, &self.config.path.events_path, &[&event]).await?;

        Ok(())
    }

    /// Verifies the maximum reference count.
    ///
    /// # Returns
    /// * `Ok(usize)` if the verification was successful.
    /// * `Err(eyre::Report)` if an error occurred during the operation.
    ///
    /// # Errors
    /// This function returns an error if the verification fails.
    pub async fn verify_refcnt_max(&self) -> Result<usize> {
        let mut count = 1;

        for host in self.hosts_config.list_hosts().await? {
            let backups = self.backups_config.get_backups(&host).await;
            count += backups.len() + 1;
        }

        Ok(count)
    }

    /// Verifies reference counts and logs events.
    ///
    /// # Arguments
    ///
    /// * `dry_run` - If true, do not modify any data.
    /// * `source` - Source of the event (CLI, daemon, etc.).
    /// * `progress_tx` - Optional channel for progress updates.
    ///
    /// # Returns
    ///
    /// * `Ok(EventRefCountInformation)` - Information about the reference count check.
    /// * `Err(eyre::Report)` if an error occurs.
    ///
    /// # Errors
    ///
    /// Returns an error if the reference count verification or event creation fails.
    pub async fn verify_refcnt(
        &self,
        dry_run: bool,
        progress_tx: Option<mpsc::Sender<FsckProgression>>,
        cancel_token: &CancellationToken,
    ) -> Result<FsckProgression> {
        info!("Starting pool reference count verification");

        // Create an intermediate channel for progress updates
        let (internal_tx, mut internal_rx) =
            mpsc::channel::<FsckProgression>(DEFAULT_CHANNEL_BUFFER_SIZE);
        let progress_thread = tokio::spawn(
            async move {
                while let Some(progress) = internal_rx.recv().await {
                    if let Some(tx) = &progress_tx {
                        if let Err(e) = tx.send(progress).await {
                            error!("Failed to send verification progress: {}", e);
                        }
                    }
                }
            }
            .in_current_span(),
        );

        let mut error_count = 0;
        let mut total_count = 0;
        let mut progress = 0;
        let mut cancelled = false;

        // Progress
        'hosts: for host in self.hosts_config.list_hosts().await? {
            let backups = self.backups_config.get_backups(&host).await;
            for backup in backups {
                if cancel_token.is_cancelled() {
                    cancelled = true;
                    break 'hosts;
                }

                debug!("Checking backup {}/{}", host, backup.id);
                let result = check_backup_integrity(
                    &host,
                    backup.id,
                    dry_run,
                    self.config.clone(),
                    self.backups_config.clone(),
                )
                .await?;

                error_count += result.error_count;
                total_count += result.total_count;
                progress += 1;

                if let Err(e) = internal_tx
                    .send(FsckProgression {
                        error_count,
                        total_count,
                        progress_current: progress,
                    })
                    .await
                {
                    error!("Failed to send verification progress: {}", e);
                }
            }

            if cancel_token.is_cancelled() {
                cancelled = true;
                break 'hosts;
            }

            debug!("Checking host {}", host);
            let result = check_host_integrity(
                &host,
                dry_run,
                self.config.clone(),
                self.backups_config.clone(),
            )
            .await?;

            error_count += result.error_count;
            total_count += result.total_count;

            progress += 1;

            if let Err(e) = internal_tx
                .send(FsckProgression {
                    error_count,
                    total_count,
                    progress_current: progress,
                })
                .await
            {
                error!("Failed to send verification progress: {}", e);
            }
        }

        // Skipped on cancel: this step recalculates and (in fix mode) saves
        // the pool-wide REFCNT from every host — not a unit anyone asked to
        // check individually, so there's no reason to still pay for it once
        // the user has stopped the run.
        if !cancelled {
            debug!("Checking pool");
            let result = check_pool_integrity(
                dry_run,
                self.config.clone(),
                self.hosts_config.clone(),
                self.backups_config.clone(),
            )
            .await?;

            error_count += result.error_count;
            total_count += result.total_count;
            progress += 1;

            if let Err(e) = internal_tx
                .send(FsckProgression {
                    error_count,
                    total_count,
                    progress_current: progress,
                })
                .await
            {
                error!("Failed to send verification progress: {}", e);
            }
        }

        // Ensure that the progress task completes
        drop(internal_tx);
        if let Err(e) = progress_thread.await {
            error!("Error in refcnt verification progression task: {}", e);
        }

        info!("Pool reference count verification completed with {total_count} total checks and {error_count} errors");
        Ok(FsckProgression {
            progress_current: total_count,
            total_count,
            error_count,
        })
    }

    /// Calculates the maximum number of unused chunks to check.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - The maximum number of unused chunks.
    /// * `Err(eyre::Report)` if an error occurs.
    ///
    /// # Errors
    ///
    /// Returns an error if loading the reference count or unused chunk data fails.
    pub async fn verify_unused_max(&self) -> Result<usize> {
        let mut pool_refcnt = Refcnt::new(&self.config.path.pool_path);
        pool_refcnt.load_refcnt(false).await;

        Ok(pool_refcnt.size())
    }

    /// Verifies unused chunks and logs events.
    ///
    /// # Arguments
    ///
    /// * `dry_run` - If true, do not modify any data.
    /// * `source` - Source of the event (CLI, daemon, etc.).
    /// * `progress_tx` - Optional channel for progress updates.
    ///
    /// # Returns
    ///
    /// * `Ok(CheckUnusedOutcome)` - Information about the unused chunk check, plus the
    ///   loaded `Refcnt` and set of hashes seen on disk — both needed by [`Self::verify_missing`].
    /// * `Err(eyre::Report)` if an error occurs.
    ///
    /// # Errors
    ///
    /// Returns an error if the unused chunk verification or event creation fails.
    pub async fn verify_unused(
        &self,
        dry_run: bool,
        progress_tx: Option<mpsc::Sender<FsckUnusedCount>>,
        cancel_token: &CancellationToken,
    ) -> Result<CheckUnusedOutcome> {
        info!("Starting unused files verification");

        let (internal_tx, mut internal_rx) = mpsc::channel(DEFAULT_CHANNEL_BUFFER_SIZE);
        let progress_thread = tokio::spawn(
            async move {
                while let Some(p) = internal_rx.recv().await {
                    if let Some(tx) = &progress_tx {
                        if let Err(e) = tx.send(p).await {
                            error!("Failed to send unused files verification progress: {}", e);
                        }
                    }
                }
            }
            .in_current_span(),
        );

        let outcome =
            check_unused(dry_run, internal_tx, self.config.clone(), cancel_token).await?;

        if let Err(e) = progress_thread.await {
            error!("Error in file list progression task: {}", e);
        }

        info!(
            "Unused files verification completed with {} in refcnt, {} in unused, and {} in nothing",
            outcome.count.in_refcnt, outcome.count.in_unused, outcome.count.in_nothing
        );
        Ok(outcome)
    }

    /// Calculates the maximum number of refcnt entries to check for missing chunks.
    ///
    /// The missing-chunk scan iterates every refcnt entry exactly once, so this is the same
    /// count as [`Self::verify_unused_max`].
    ///
    /// # Errors
    ///
    /// Returns an error if loading the reference count data fails.
    pub async fn verify_missing_max(&self) -> Result<usize> {
        self.verify_unused_max().await
    }

    /// Identifies refcnt entries that reference a chunk absent from disk.
    ///
    /// Must be called with the `refcnt` and `seen` set produced by a prior
    /// [`Self::verify_unused`] call — it reuses them instead of re-walking or `stat`-ing the
    /// pool.
    ///
    /// # Errors
    ///
    /// Returns an error if the durable "missing chunks" snapshot cannot be written.
    pub async fn verify_missing(
        &self,
        refcnt: &Refcnt,
        seen: &HashSet<[u8; 32]>,
        progress_tx: Option<mpsc::Sender<FsckMissingCount>>,
        cancel_token: &CancellationToken,
    ) -> Result<FsckMissingCount> {
        info!("Starting missing chunks verification");

        let (internal_tx, mut internal_rx) = mpsc::channel(DEFAULT_CHANNEL_BUFFER_SIZE);
        let progress_thread = tokio::spawn(
            async move {
                while let Some(p) = internal_rx.recv().await {
                    if let Some(tx) = &progress_tx {
                        if let Err(e) = tx.send(p).await {
                            error!("Failed to send missing chunks verification progress: {}", e);
                        }
                    }
                }
            }
            .in_current_span(),
        );

        let result = check_missing(
            refcnt,
            seen,
            internal_tx,
            self.config.clone(),
            cancel_token,
        )
        .await?;

        if let Err(e) = progress_thread.await {
            error!("Error in missing chunks progression task: {}", e);
        }

        info!(
            "Missing chunks verification completed with {} checked and {} missing",
            result.checked, result.missing
        );
        Ok(result)
    }

    /// Calculates the maximum number of chunks to check for integrity.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Vec<u8>>)` - A list of chunk identifiers.
    /// * `Err(eyre::Report)` if an error occurs.
    ///
    /// # Errors
    ///
    /// Returns an error if loading the reference count or unused chunk data fails.
    pub async fn verify_chunk_max(&self) -> Result<Vec<Vec<u8>>> {
        let mut pool_refcnt = Refcnt::new(&self.config.path.pool_path);
        pool_refcnt.load_refcnt(false).await;

        let chunks = pool_refcnt
            .list_refcnt()
            .map(|refcnt| refcnt.sha256.clone())
            .collect::<Vec<_>>();

        Ok(chunks)
    }

    /// Verifies chunk integrity and logs events.
    ///
    /// # Arguments
    ///
    /// * `source` - Source of the event (CLI, daemon, etc.).
    /// * `progress_tx` - Optional channel for progress updates.
    ///
    /// # Returns
    ///
    /// * `Ok(EventRefCountInformation)` - Information about the chunk integrity check.
    /// * `Err(eyre::Report)` if an error occurs.
    ///
    /// # Errors
    ///
    /// Returns an error if the chunk integrity verification or event creation fails.
    pub async fn verify_chunk(
        &self,
        progress_tx: Option<mpsc::Sender<FsckProgression>>,
        cancel_token: &CancellationToken,
    ) -> Result<FsckProgression> {
        info!("Starting chunk verification");
        // Create an intermediate channel for progress updates
        let (internal_tx, mut internal_rx) =
            mpsc::channel::<FsckProgression>(DEFAULT_CHANNEL_BUFFER_SIZE);
        let progress_thread = tokio::spawn(
            async move {
                while let Some(progress) = internal_rx.recv().await {
                    if let Some(tx) = &progress_tx {
                        if let Err(e) = tx.send(progress).await {
                            error!("Failed to send chunk verification progress: {}", e);
                        }
                    }
                }
            }
            .in_current_span(),
        );

        let chunks = self.verify_chunk_max().await?;
        let mut error_count = 0;
        let mut total_count = 0;

        for refcnt in chunks {
            if cancel_token.is_cancelled() {
                break;
            }

            let wrapper = PoolChunkWrapper::new(&self.config.path.pool_path, Some(&refcnt));

            let is_valid = wrapper
                .check_chunk_information(self.config.chunk_algorithm)
                .await;

            if !is_valid.is_ok_and(|f| f) {
                error_count += 1;
            }

            total_count += 1;

            if let Err(e) = internal_tx
                .send(FsckProgression {
                    error_count,
                    total_count,
                    progress_current: total_count,
                })
                .await
            {
                error!("Failed to send chunk verification progress: {}", e);
            }
        }

        // Ensure that the progress task completes
        drop(internal_tx);
        if let Err(e) = progress_thread.await {
            error!("Error in chunk verification progression task: {}", e);
        }

        info!(
            "Chunk verification completed with {total_count} total checks and {error_count} errors"
        );
        Ok(FsckProgression {
            progress_current: total_count,
            total_count,
            error_count,
        })
    }
}

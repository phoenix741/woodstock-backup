use std::{sync::Arc, time::SystemTime};
use tokio::fs::read_dir;
use tokio::sync::mpsc;

use eyre::Result;
use tracing::{debug, error, info, Instrument};
use uuid::Uuid;

use crate::{
    config::{Backups, Configuration, Hosts, DEFAULT_CHANNEL_BUFFER_SIZE},
    events::append_events,
    pool::{
        check_backup_integrity, check_host_integrity, check_pool_integrity, check_unused, data,
        FsckUnusedCount,
    },
    woodstock::event::Information,
    Event, EventPoolInformation, EventSource, EventStatus, EventStep, EventType,
};

/// # Pool Filesystem Check (fsck) Module
///
/// This module provides logic for verifying and repairing the integrity of the Woodstock backup pool.
/// It includes logical index verification, unused chunk detection, chunk integrity verification, and event logging.
///
/// ## Main Structures
///
/// - [`FsckProgression`]: Tracks progress and errors during fsck operations.
/// - [`PoolProgression`]: Tracks progress and statistics for pool operations.
/// - [`PoolFsck`]: Main struct for running fsck operations on the backup pool.
///
/// ## Main Methods
///
/// - [`PoolFsck::verify_index_max`]: Calculates the maximum number of logical index checks.
/// - [`PoolFsck::verify_index`]: Verifies logical chunk visibility and logs events.
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
/// - [`Event`]: For related pool and event operations
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

    /// Creates a new event to mark the end of a pool integrity operation.
    ///
    /// # Arguments
    /// * `id` - The identifier of the event.
    /// * `information` - Additional information about the pool integrity operation.
    ///
    /// # Returns
    /// * `Ok(())` if the event was successfully created.
    /// * `Err(eyre::Report)` if an error occurred during the operation.
    ///
    /// # Errors
    /// This function returns an error if the event creation fails.
    pub async fn create_event_fsck_end(
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

    /// Calculates the maximum number of logical index checks.
    ///
    /// # Returns
    /// * `Ok(usize)` if the verification was successful.
    /// * `Err(eyre::Report)` if an error occurred during the operation.
    ///
    /// # Errors
    /// This function returns an error if the verification fails.
    pub async fn verify_index_max(&self) -> Result<usize> {
        let mut count = 1;

        for host in self.hosts_config.list_hosts().await? {
            let backups = self.backups_config.get_backups(&host).await;
            count += backups.len() + 1;
        }

        Ok(count)
    }

    pub async fn verify_segments_max(&self) -> Result<usize> {
        let segments_path = data::segments_directory_path(&self.config.path.pool_path);
        if !segments_path.exists() {
            return Ok(0);
        }

        let mut entries = read_dir(&segments_path).await?;
        let mut count = 0;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().is_some_and(|extension| extension == "seg") {
                count += 1;
            }
        }

        Ok(count)
    }

    pub async fn verify_segments(
        &self,
        progress_tx: Option<mpsc::Sender<FsckProgression>>,
    ) -> Result<FsckProgression> {
        info!("Starting Pool V3 segment verification");

        let (internal_tx, mut internal_rx) =
            mpsc::channel::<FsckProgression>(DEFAULT_CHANNEL_BUFFER_SIZE);
        let progress_thread = tokio::spawn(
            async move {
                while let Some(progress) = internal_rx.recv().await {
                    if let Some(tx) = &progress_tx {
                        if let Err(e) = tx.send(progress).await {
                            error!("Failed to send segment verification progress: {}", e);
                        }
                    }
                }
            }
            .in_current_span(),
        );

        let segments_path = data::segments_directory_path(&self.config.path.pool_path);
        let mut error_count = 0;
        let mut total_count = 0;

        if segments_path.exists() {
            let mut entries = read_dir(&segments_path).await?;
            let mut segment_paths = Vec::new();

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.extension().is_some_and(|extension| extension == "seg") {
                    segment_paths.push(path);
                }
            }

            segment_paths.sort();

            for path in segment_paths {
                let display_path = path.display().to_string();
                match data::open_existing_segment(&path).await {
                    Ok(segment_file) => match segment_file.chunks().await {
                        Ok(_) => {}
                        Err(err) => {
                            error_count += 1;
                            error!("Pool V3 segment {} scan failed: {}", display_path, err);
                        }
                    },
                    Err(err) => {
                        error_count += 1;
                        error!(
                            "Pool V3 segment {} header validation failed: {}",
                            display_path,
                            err
                        );
                    }
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
                    error!("Failed to send segment verification progress: {}", e);
                }
            }
        }

        drop(internal_tx);
        if let Err(e) = progress_thread.await {
            error!("Error in segment verification progression task: {}", e);
        }

        info!(
            "Segment verification completed with {total_count} total checks and {error_count} errors"
        );
        Ok(FsckProgression {
            progress_current: total_count,
            total_count,
            error_count,
        })
    }

    /// Verifies logical chunk visibility and logs events.
    ///
    /// # Arguments
    ///
    /// * `dry_run` - If true, do not modify any data.
    /// * `source` - Source of the event (CLI, daemon, etc.).
    /// * `progress_tx` - Optional channel for progress updates.
    ///
    /// # Returns
    ///
    /// * `Ok(EventRefCountInformation)` - Information about the logical index check.
    /// * `Err(eyre::Report)` if an error occurs.
    ///
    /// # Errors
    ///
    /// Returns an error if the logical index verification or event creation fails.
    pub async fn verify_index(
        &self,
        dry_run: bool,
        progress_tx: Option<mpsc::Sender<FsckProgression>>,
    ) -> Result<FsckProgression> {
        info!("Starting Pool V3 logical index verification");

        // Create an intermediate channel for progress updates
        let (internal_tx, mut internal_rx) =
            mpsc::channel::<FsckProgression>(DEFAULT_CHANNEL_BUFFER_SIZE);
        let progress_thread = tokio::spawn(
            async move {
                while let Some(progress) = internal_rx.recv().await {
                    if let Some(tx) = &progress_tx {
                        if let Err(e) = tx.send(progress).await {
                            error!("Failed to send logical index verification progress: {}", e);
                        }
                    }
                }
            }
            .in_current_span(),
        );

        let mut error_count = 0;
        let mut total_count = 0;
        let mut progress = 0;

        // Progress
        for host in self.hosts_config.list_hosts().await? {
            let backups = self.backups_config.get_backups(&host).await;
            for backup in backups {
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
                    error!("Failed to send logical index verification progress: {}", e);
                }
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
                error!("Failed to send logical index verification progress: {}", e);
            }
        }

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
            error!("Failed to send logical index verification progress: {}", e);
        }

        // Ensure that the progress task completes
        drop(internal_tx);
        if let Err(e) = progress_thread.await {
            error!(
                "Error in logical index verification progression task: {}",
                e
            );
        }

        info!("Pool V3 logical index verification completed with {total_count} total checks and {error_count} errors");
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
    /// Returns an error if loading the materialized index or unused chunk data fails.
    pub async fn verify_unused_max(&self) -> Result<usize> {
        let index_path = data::pool_index_path(&self.config.path.pool_path);
        if !index_path.exists() {
            return Ok(0);
        }

        let index = data::open_pool_index(&self.config.path.pool_path)?;
        let count = index
            .list_segments()?
            .into_iter()
            .map(|segment| segment.chunk_count as usize)
            .sum();
        Ok(count)
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
    /// * `Ok(FsckUnusedCount)` - Information about the unused chunk check.
    /// * `Err(eyre::Report)` if an error occurs.
    ///
    /// # Errors
    ///
    /// Returns an error if the unused chunk verification or event creation fails.
    pub async fn verify_unused(
        &self,
        dry_run: bool,
        progress_tx: Option<mpsc::Sender<FsckUnusedCount>>,
    ) -> Result<FsckUnusedCount> {
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

        let result = check_unused(dry_run, internal_tx, self.config.clone()).await?;

        if let Err(e) = progress_thread.await {
            error!("Error in file list progression task: {}", e);
        }

        info!(
            "Unused files verification completed with {} visible, {} unused, {} unindexed, and {} missing",
            result.in_refcnt, result.in_unused, result.in_nothing, result.missing
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
    /// Returns an error if loading the materialized index fails.
    pub async fn verify_chunk_max(&self) -> Result<Vec<Vec<u8>>> {
        let index_path = data::pool_index_path(&self.config.path.pool_path);
        if !index_path.exists() {
            return Ok(Vec::new());
        }

        let index = data::open_pool_index(&self.config.path.pool_path)?;
        let chunks = index
            .list_chunks()?
            .into_iter()
            .map(|chunk| chunk.hash)
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
    ) -> Result<FsckProgression> {
        info!("Starting Pool V3 chunk verification");
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

        let mut error_count = 0;
        let mut total_count = 0;

        let index_path = data::pool_index_path(&self.config.path.pool_path);
        if index_path.exists() {
            let index = data::open_pool_index(&self.config.path.pool_path)?;
            for chunk in index.list_chunks()? {
                let segment = index.get_segment(chunk.segment_id)?;

                let is_valid = if let Some(segment) = segment {
                    let path = data::resolve_segment_path(&self.config.path.pool_path, &segment);
                    if !path.exists() {
                        false
                    } else {
                        match data::open_existing_segment(&path).await {
                            Ok(segment_file) => {
                                match segment_file.read_chunk_at(chunk.offset).await {
                                    Ok((entry, _)) => {
                                        entry.hash == chunk.hash
                                            && entry.size == chunk.size
                                            && entry.compressed_size == chunk.compressed_size
                                            && entry.compression_format == chunk.compression_format
                                            && entry.chunk_header_size == chunk.chunk_header_size
                                    }
                                    Err(_) => false,
                                }
                            }
                            Err(_) => false,
                        }
                    }
                } else {
                    false
                };

                if !is_valid {
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

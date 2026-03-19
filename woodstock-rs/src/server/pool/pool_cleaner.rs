use std::{path::PathBuf, sync::Arc, time::SystemTime};
use tokio::sync::mpsc;

use eyre::Result;
use tracing::error;
use uuid::Uuid;

use crate::{
    config::Configuration, events::append_events, pool::PoolManager, woodstock::event::Information,
    Event, EventPoolCleanedInformation, EventSource, EventStatus, EventStep, EventType,
};

/// Progress payload sent by the pool cleaner to CLI and job consumers.
///
/// In Pool V2 this tracks the cleanup of unused chunk files. In Pool V3 the same structure is
/// reused to expose incremental compaction progress while preserving the existing public contract.
#[derive(Clone, Debug)]
pub struct PoolProgression {
    /// Current number of processed items.
    pub progress_current: usize,
    /// Total number of items expected for the current run.
    pub file_count: usize,
    /// Number of bytes processed so far.
    pub file_size: u64,
    /// Number of bytes reclaimed or cleaned so far.
    pub compressed_file_size: u64,
}

#[derive(Clone)]
pub struct PoolCleaner {
    /// The configuration used by the pool cleaner to determine cleaning parameters and behavior.
    config: Arc<Configuration>,
}

impl PoolCleaner {
    /// Creates a new instance of `PoolCleaner`.
    ///
    /// # Arguments
    /// * `config` - The configuration for the pool cleaner.
    ///
    /// # Returns
    ///
    /// A new instance of `PoolCleaner`.
    #[must_use]
    pub fn new(config: Arc<Configuration>) -> Self {
        PoolCleaner { config }
    }

    /// Creates a start event for the pool cleaning process.
    ///
    /// # Arguments
    /// * `event_type` - The type of the event.
    /// * `source` - The source of the event.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` containing the event ID if the event creation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during event creation.
    async fn create_event_start(
        &self,
        event_type: EventType,
        source: EventSource,
    ) -> Result<Vec<u8>> {
        let id = Uuid::new_v4();
        let id = id.as_bytes();

        let event = Event {
            id: id.to_vec(),
            event_type: event_type as i32,
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

    /// Creates an end event for the pool cleaning process.
    ///
    /// # Arguments
    /// * `id` - The ID of the event.
    /// * `source` - The source of the event.
    /// * `information` - Information about the pool cleaning process.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the event creation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during event creation.
    async fn create_event_cleaned_end(
        &self,
        id: &[u8],
        source: EventSource,
        information: EventPoolCleanedInformation,
    ) -> Result<()> {
        let event = Event {
            id: id.to_vec(),
            event_type: EventType::PoolCleaned as i32,
            step: EventStep::End as i32,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
            source: source as i32,
            user: String::new(),
            error_messages: Vec::new(),
            status: EventStatus::Success as i32,

            information: Some(Information::PoolCleaned(information)),
        };

        append_events(&self.config, &self.config.path.events_path, &[&event]).await?;

        Ok(())
    }

    /// Calculates the maximum number of unused files in the pool.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` containing the count of unused files if the calculation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the calculation.
    ///
    /// # Errors
    ///
    /// Returns an error if the unused files cannot be loaded or counted due to I/O or data issues.
    pub async fn clean_unused_max(&self) -> Result<usize> {
        PoolManager::new(self.config.clone()).compact_pool_v3_max()
    }

    /// Cleans unused files from the pool.
    ///
    /// On a Pool V3 layout, this method runs segment compaction instead of legacy unused-file
    /// deletion and emits incremental progress snapshots through the existing `PoolProgression`
    /// channel.
    ///
    /// # Arguments
    /// * `target` - An optional target path for cleaning.
    /// * `source` - The source of the event.
    /// * `progress_tx` - An optional channel for sending progress updates.
    ///
    /// # Returns
    ///
    /// * `Ok(EventPoolCleanedInformation)` containing information about the cleaned pool if the cleaning succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the cleaning process.
    ///
    /// # Errors
    ///
    /// Returns an error if the cleaning process fails.
    pub async fn clean_unused_pool(
        &self,
        target: Option<PathBuf>,
        source: EventSource,
        progress_tx: Option<mpsc::Sender<PoolProgression>>,
    ) -> Result<EventPoolCleanedInformation> {
        let pool_manager = PoolManager::new(self.config.clone());
        let total = pool_manager.compact_pool_v3_max()?;
        let id = self
            .create_event_start(EventType::PoolCleaned, source)
            .await?;
        let compacted = pool_manager
            .compact_pool_v3_with_progress(target.as_deref(), |progress| {
                if let Some(tx) = &progress_tx {
                    if let Err(error) = tx.try_send(PoolProgression {
                        progress_current: progress.processed_segments,
                        file_count: progress.total_segments.max(total),
                        file_size: progress.rewritten_bytes,
                        compressed_file_size: progress.reclaimed_size,
                    }) {
                        error!("Failed to send pool compaction progress: {}", error);
                    }
                }
            })
            .await?;

        if let Some(tx) = &progress_tx {
            if let Err(error) = tx.try_send(PoolProgression {
                progress_current: compacted.removed_segments as usize,
                file_count: total,
                file_size: compacted.reclaimed_size,
                compressed_file_size: compacted.reclaimed_size,
            }) {
                error!("Failed to send final pool compaction progress: {}", error);
            }
        }

        let informations = EventPoolCleanedInformation {
            count: compacted.removed_segments,
            size: compacted.reclaimed_size,
        };
        self.create_event_cleaned_end(&id, source, informations.clone())
            .await?;

        Ok(informations)
    }
}

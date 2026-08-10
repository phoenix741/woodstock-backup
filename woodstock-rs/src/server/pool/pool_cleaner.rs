use std::{
    path::PathBuf,
    sync::{atomic::AtomicU64, atomic::Ordering, Arc},
    time::SystemTime,
};
use tokio::sync::mpsc;

use eyre::Result;
use tracing::{error, Instrument};
use uuid::Uuid;

use crate::{
    config::{Configuration, DEFAULT_CHANNEL_BUFFER_SIZE},
    events::append_events,
    pool::Refcnt,
    woodstock::event::Information,
    Event, EventPoolCleanedInformation, EventSource, EventStatus, EventStep, EventType, PoolUnused,
};

#[derive(Clone, Debug)]
pub struct PoolProgression {
    pub progress_current: usize,
    pub file_count: usize,
    pub file_size: u64,
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
        let mut refcnt = Refcnt::new(&self.config.path.pool_path);
        refcnt.load_unused().await;

        Ok(refcnt.list_refcnt().count())
    }

    /// Cleans unused files from the pool.
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
        let id = self
            .create_event_start(EventType::PoolCleaned, source)
            .await?;

        let mut refcnt = Refcnt::new(&self.config.path.pool_path);
        refcnt.load_unused().await;

        let total_size = AtomicU64::new(0);
        let total_compressed_size = Arc::new(AtomicU64::new(0));
        let total = refcnt.size();

        let total_compressed_size_progress = total_compressed_size.clone();
        let removed_hashes = Arc::new(tokio::sync::Mutex::new(Vec::<Vec<u8>>::new()));
        let removed_hashes_progress = removed_hashes.clone();

        let (internal_tx, mut internal_rx) =
            mpsc::channel::<Option<PoolUnused>>(DEFAULT_CHANNEL_BUFFER_SIZE);
        let progress_thread = tokio::spawn(
            async move {
                let mut count = 0;
                while let Some(unused) = internal_rx.recv().await {
                    let compressed_size = unused
                        .clone()
                        .map(|f| f.compressed_size)
                        .unwrap_or_default();
                    let size = unused.clone().map(|f| f.size).unwrap_or_default();
                    if let Some(unused) = &unused {
                        removed_hashes_progress
                            .lock()
                            .await
                            .push(unused.sha256.clone());
                    }

                    total_compressed_size_progress.fetch_add(compressed_size, Ordering::SeqCst);
                    total_size.fetch_add(size, Ordering::SeqCst);
                    count += 1;

                    if let Some(tx) = &progress_tx {
                        if let Err(e) = tx
                            .send(PoolProgression {
                                progress_current: count,
                                file_count: total,
                                file_size: total_size.load(Ordering::SeqCst),
                                compressed_file_size: total_compressed_size_progress
                                    .load(Ordering::SeqCst),
                            })
                            .await
                        {
                            error!("Failed to send unused files cleanup progress: {}", e);
                        }
                    }
                }
            }
            .in_current_span(),
        );

        refcnt
            .remove_unused_files(
                &self.config.path.pool_path,
                target,
                internal_tx,
                self.config.compression_format.clone(),
            )
            .await?;

        if let Err(e) = progress_thread.await {
            error!("Error in file list progression task: {}", e);
        }

        let informations = EventPoolCleanedInformation {
            count: total as u64,
            size: total_compressed_size.load(Ordering::SeqCst),
            removed_hashes: removed_hashes.lock().await.clone(),
        };
        self.create_event_cleaned_end(&id, source, informations.clone())
            .await?;

        Ok(informations)
    }
}

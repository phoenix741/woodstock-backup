use std::{path::PathBuf, sync::atomic::AtomicU64, time::SystemTime};

use eyre::Result;
use log::debug;
use std::sync::atomic::{AtomicUsize, Ordering};
use uuid::Uuid;

use crate::{
    config::{Backups, Context, Hosts},
    events::append_events,
    pool::{
        check_backup_integrity, check_host_integrity, check_pool_integrity, check_unused,
        PoolChunkWrapper, Refcnt,
    },
    utils::lock::PoolLock,
    woodstock::event::Information,
    Event, EventPoolCleanedInformation, EventPoolInformation, EventRefCountInformation,
    EventSource, EventStatus, EventStep, EventType,
};

#[derive(Clone, Debug)]
pub struct FsckProgression {
    pub error_count: usize,
    pub total_count: usize,

    pub progress_current: usize,
}

#[derive(Clone, Debug)]
pub struct PoolProgression {
    pub progress_current: usize,

    pub file_count: usize,
    pub file_size: u64,
    pub compressed_file_size: u64,
}

#[derive(Clone)]
pub struct PoolFsck {
    source: EventSource,
    context: Context,
}

impl PoolFsck {
    pub fn new(ctxt: &Context) -> Self {
        PoolFsck {
            source: ctxt.source,
            context: ctxt.clone(),
        }
    }

    async fn create_event_start(&self, event_type: EventType) -> Result<Vec<u8>> {
        let id = Uuid::new_v4();
        let id = id.as_bytes();

        let event = Event {
            id: id.to_vec(),
            r#type: event_type as i32,
            step: EventStep::Start as i32,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
            source: self.source as i32,
            user: None,
            error_messages: Vec::new(),
            status: EventStatus::None as i32,

            information: None,
        };

        append_events(&self.context.config.path.events_path, &[&event]).await?;

        Ok(id.to_vec())
    }

    async fn create_event_refcnt_end(
        &self,
        id: &[u8],
        information: EventRefCountInformation,
    ) -> Result<()> {
        let event = Event {
            id: id.to_vec(),
            r#type: EventType::RefcntChecked as i32,
            step: EventStep::End as i32,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
            source: self.source as i32,
            user: None,
            error_messages: Vec::new(),
            status: if information.error > 0 {
                EventStatus::GenericError
            } else {
                EventStatus::Success
            } as i32,

            information: Some(Information::Refcnt(information)),
        };

        append_events(&self.context.config.path.events_path, &[&event]).await?;

        Ok(())
    }

    async fn create_event_pool_end(
        &self,
        id: &[u8],
        information: EventPoolInformation,
    ) -> Result<()> {
        let event = Event {
            id: id.to_vec(),
            r#type: EventType::PoolChecked as i32,
            step: EventStep::End as i32,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
            source: self.source as i32,
            user: None,
            error_messages: Vec::new(),
            status: if information.in_nothing > 0 || information.missing > 0 {
                EventStatus::GenericError
            } else {
                EventStatus::Success
            } as i32,

            information: Some(Information::Pool(information)),
        };

        append_events(&self.context.config.path.events_path, &[&event]).await?;

        Ok(())
    }

    async fn create_event_chunk_end(
        &self,
        id: &[u8],
        information: EventRefCountInformation,
    ) -> Result<()> {
        let event = Event {
            id: id.to_vec(),
            r#type: EventType::PoolChecked as i32,
            step: EventStep::End as i32,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
            source: self.source as i32,
            user: None,
            error_messages: Vec::new(),
            status: if information.error > 0 {
                EventStatus::GenericError
            } else {
                EventStatus::Success
            } as i32,

            information: Some(Information::Refcnt(information)),
        };

        append_events(&self.context.config.path.events_path, &[&event]).await?;

        Ok(())
    }

    async fn create_event_cleaned_end(
        &self,
        id: &[u8],
        information: EventPoolCleanedInformation,
    ) -> Result<()> {
        let event = Event {
            id: id.to_vec(),
            r#type: EventType::PoolCleaned as i32,
            step: EventStep::End as i32,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
            source: self.source as i32,
            user: None,
            error_messages: Vec::new(),
            status: EventStatus::Success as i32,

            information: Some(Information::PoolCleaned(information)),
        };

        append_events(&self.context.config.path.events_path, &[&event]).await?;

        Ok(())
    }

    pub async fn verify_refcnt_max(&self) -> Result<usize> {
        let mut count = 1;

        // Calculate max
        let hosts = Hosts::new(&self.context);
        let backups = Backups::new(&self.context);

        for host in hosts.list_hosts().await? {
            let backups = backups.get_backups(&host).await;
            count += backups.len() + 1;
        }

        Ok(count)
    }

    pub async fn verify_refcnt(
        &self,
        dry_run: bool,
        callback: &impl Fn(&FsckProgression),
    ) -> Result<EventRefCountInformation> {
        let id = self.create_event_start(EventType::RefcntChecked).await?;

        let path = &self.context.config.path.pool_path;
        let _lock = PoolLock::new(path).lock().await?;

        let hosts = Hosts::new(&self.context);
        let backups = Backups::new(&self.context);

        let mut error_count = 0;
        let mut total_count = 0;
        let mut progress = 0;

        // Progress
        for host in hosts.list_hosts().await? {
            let backups = backups.get_backups(&host).await;
            for backup in backups {
                debug!("Checking backup {}/{}", host, backup.number);
                let result =
                    check_backup_integrity(&host, backup.number, dry_run, &self.context).await?;

                error_count += result.error_count;
                total_count += result.total_count;
                progress += 1;

                callback(&FsckProgression {
                    error_count,
                    total_count,
                    progress_current: progress,
                });
            }

            debug!("Checking host {}", host);
            let result = check_host_integrity(&host, dry_run, &self.context).await?;

            error_count += result.error_count;
            total_count += result.total_count;

            progress += 1;

            callback(&FsckProgression {
                error_count,
                total_count,
                progress_current: progress,
            });
        }

        debug!("Checking pool");
        let result = check_pool_integrity(dry_run, &self.context).await?;

        error_count += result.error_count;
        total_count += result.total_count;

        progress += 1;

        callback(&FsckProgression {
            error_count,
            total_count,
            progress_current: progress,
        });

        let information = EventRefCountInformation {
            fix: !dry_run,

            count: total_count as u64,
            error: error_count as u64,
        };
        self.create_event_refcnt_end(&id, information).await?;

        Ok(information)
    }

    pub async fn verify_unused_max(&self) -> Result<usize> {
        let mut pool_refcnt = Refcnt::new(&self.context.config.path.pool_path);
        pool_refcnt.load_refcnt(false).await;
        pool_refcnt.load_unused().await;

        let total = pool_refcnt.list_unused().count() + pool_refcnt.list_refcnt().count();

        Ok(total)
    }

    pub async fn verify_unused(
        &self,
        dry_run: bool,
        callback: &impl Fn(&PoolProgression),
    ) -> Result<EventPoolInformation> {
        let id = self.create_event_start(EventType::PoolChecked).await?;

        let _lock = PoolLock::new(&self.context.config.path.pool_path)
            .lock()
            .await?;

        let count = AtomicUsize::new(0);

        let result = check_unused(
            dry_run,
            &|p| {
                let count = count.fetch_add(p, Ordering::SeqCst);

                callback(&PoolProgression {
                    progress_current: count + p,
                    file_count: 0,
                    file_size: 0,
                    compressed_file_size: 0,
                });
            },
            &self.context,
        )
        .await?;

        let information = EventPoolInformation {
            fix: !dry_run,

            in_nothing: result.in_nothing as u64,
            in_refcnt: result.in_refcnt as u64,
            in_unused: result.in_unused as u64,
            missing: result.missing as u64,
        };

        self.create_event_pool_end(&id, information).await?;

        Ok(information)
    }

    pub async fn verify_chunk_max(&self) -> Result<Vec<Vec<u8>>> {
        let mut pool_refcnt = Refcnt::new(&self.context.config.path.pool_path);
        pool_refcnt.load_refcnt(false).await;
        pool_refcnt.load_unused().await;

        let mut chunks = pool_refcnt
            .list_refcnt()
            .map(|refcnt| refcnt.sha256.clone())
            .collect::<Vec<_>>();
        chunks.extend(
            pool_refcnt
                .list_unused()
                .map(|unused| unused.sha256.clone()),
        );

        Ok(chunks)
    }

    pub async fn verify_chunk(
        &self,
        callback: &impl Fn(&FsckProgression),
    ) -> Result<EventRefCountInformation> {
        let id = self.create_event_start(EventType::ChecksumChecked).await?;

        let _lock = PoolLock::new(&self.context.config.path.pool_path)
            .lock()
            .await?;

        let chunks = self.verify_chunk_max().await?;
        let mut error_count = 0;
        let mut total_count = 0;

        for refcnt in chunks {
            let wrapper = PoolChunkWrapper::new(&self.context.config.path.pool_path, Some(&refcnt));

            let is_valid = wrapper.check_chunk_information().await?;
            if !is_valid {
                error_count += 1;
            }

            total_count += 1;

            callback(&FsckProgression {
                error_count,
                total_count,

                progress_current: total_count,
            });
        }

        let informations = EventRefCountInformation {
            fix: false,

            count: total_count as u64,
            error: error_count as u64,
        };
        self.create_event_chunk_end(&id, informations).await?;

        Ok(informations)
    }

    pub async fn clean_unused_max(&self) -> Result<usize> {
        let mut refcnt = Refcnt::new(&self.context.config.path.pool_path);
        refcnt.load_unused().await;

        Ok(refcnt.list_unused().count())
    }

    pub async fn clean_unused_pool(
        &self,
        target: Option<PathBuf>,
        callback: &impl Fn(&PoolProgression),
    ) -> Result<EventPoolCleanedInformation> {
        let id = self.create_event_start(EventType::PoolCleaned).await?;

        let _lock = PoolLock::new(&self.context.config.path.pool_path)
            .lock()
            .await?;

        let mut refcnt = Refcnt::new(&self.context.config.path.pool_path);
        refcnt.load_unused().await;

        let count = AtomicUsize::new(0);
        let total_size = AtomicU64::new(0);
        let total_compressed_size = AtomicU64::new(0);
        let total = refcnt.list_unused().count();

        refcnt
            .remove_unused_files(&self.context.config.path.pool_path, target, &|unused| {
                let compressed_size = unused
                    .clone()
                    .map(|f| f.compressed_size)
                    .unwrap_or_default();
                let size = unused.clone().map(|f| f.size).unwrap_or_default();

                total_compressed_size.fetch_add(compressed_size, Ordering::SeqCst);
                total_size.fetch_add(size, Ordering::SeqCst);
                let count = count.fetch_add(1, Ordering::SeqCst);

                callback(&PoolProgression {
                    progress_current: count + 1,
                    file_count: total,
                    file_size: total_size.load(Ordering::SeqCst),
                    compressed_file_size: total_compressed_size.load(Ordering::SeqCst),
                });
            })
            .await?;

        let informations = EventPoolCleanedInformation {
            count: total as u64,
            size: total_compressed_size.load(Ordering::SeqCst),
        };
        self.create_event_cleaned_end(&id, informations).await?;

        Ok(informations)
    }
}

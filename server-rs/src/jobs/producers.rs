//! Producteurs de jobs (scanner wakeup, événements, nightly)

use std::sync::Arc;

use apalis::prelude::*;
use apalis_redis::RedisStorage;
use tracing::instrument;
use uuid::Uuid;
use woodstock::config::{Backups, Hosts};

use crate::jobs::progress::{JobKind, ProgressPublisher};
use crate::jobs::types::*;

/// Contrat minimal pour les producteurs, pour faciliter les tests
#[derive(Clone)]
pub struct Producers {
    pub hosts: Arc<Hosts>,
    pub backups: Arc<Backups>,
    pub schedule_storage: RedisStorage<ScheduleQueueJob>,
    pub backup_storage: RedisStorage<BackupQueueJob>,
    pub interactive_storage: RedisStorage<RestoreJobData>,
    pub maintenance_storage: RedisStorage<MaintenanceJobData>,
    pub progress_publisher: ProgressPublisher,
    pub redis_client: redis::Client,
}

impl Producers {
    pub fn new(
        hosts: Arc<Hosts>,
        backups: Arc<Backups>,
        schedule_storage: RedisStorage<ScheduleQueueJob>,
        backup_storage: RedisStorage<BackupQueueJob>,
        interactive_storage: RedisStorage<RestoreJobData>,
        maintenance_storage: RedisStorage<MaintenanceJobData>,
        progress_publisher: ProgressPublisher,
        redis_client: redis::Client,
    ) -> Self {
        Self {
            hosts,
            backups,
            schedule_storage,
            backup_storage,
            interactive_storage,
            maintenance_storage,
            progress_publisher,
            redis_client,
        }
    }

    /// Enqueue un backup pour un host, en respectant l'unicité (parité TS)
    /// Implémentation: clé garde Redis backup::{host} via SET NX PX 30s, puis push si acquise.
    #[instrument(skip(self))]
    pub async fn enqueue_backup_unique(
        &mut self,
        host: &str,
        force: bool,
    ) -> Result<Option<String>, Error> {
        // Clé d'unicité identique à TS
        let key = backup_unique_key(host);
        // Fenêtre de 30 secondes pour éviter le rebond
        let ttl_ms: u64 = 30_000;
        let mut conn = self
            .redis_client
            .get_multiplexed_tokio_connection()
            .await
            .map_err(|e| Error::from(Box::<dyn std::error::Error + Send + Sync>::from(e)))?;

        let ok: Option<String> = redis::cmd("SET")
            .arg(&[&key, "1", "NX", "PX", &ttl_ms.to_string()])
            .query_async(&mut conn)
            .await
            .map_err(|e| Error::from(Box::<dyn std::error::Error + Send + Sync>::from(e)))?;

        if ok.is_some() {
            let config =
                self.hosts.get_host(host).await.map_err(|e| {
                    Error::from(Box::<dyn std::error::Error + Send + Sync>::from(e))
                })?;
            let (next_number, previous_id) = match self.backups.get_last_backup(&host).await {
                Some(last) => (last.number + 1, Some(last.id)),
                None => (0, None),
            };

            let backup_job_data = BackupJobData {
                host: host.to_string(),
                config,
                id: Uuid::now_v7(),
                number: next_number,
                previous_id,
                ip: None,
                start_date: None,
                force,
            };
            let data = BackupQueueJob::Save(backup_job_data.clone());
            let parts =
                self.backup_storage.push(data).await.map_err(|e| {
                    Error::from(Box::<dyn std::error::Error + Send + Sync>::from(e))
                })?;

            self.progress_publisher
                .create_job(
                    &parts.task_id.to_string(),
                    JobKind::with_backup(backup_job_data),
                    Some(&host),
                )
                .await
                .map_err(|e| Error::from(Box::<dyn std::error::Error + Send + Sync>::from(e)))?;

            Ok(Some(parts.task_id.to_string()))
        } else {
            // Rien à faire, un job équivalent a déjà été enqueued récemment
            Ok(None)
        }
    }

    /// Enqueue un job de suppression de backup
    #[instrument(skip(self))]
    pub async fn enqueue_remove(
        &mut self,
        host: &str,
        backup_id: Uuid,
        backup_number: usize,
    ) -> Result<String, Error> {
        let remove_job_data = RemoveJobData {
            host: host.to_string(),
            id: backup_id,
            number: backup_number,
            ..Default::default()
        };
        let data = BackupQueueJob::Remove(remove_job_data.clone());
        let parts = self
            .backup_storage
            .push(data)
            .await
            .map_err(|e| Error::from(Box::<dyn std::error::Error + Send + Sync>::from(e)))?;

        self.progress_publisher
            .create_job(
                &parts.task_id.to_string(),
                JobKind::with_remove(remove_job_data),
                Some(&host),
            )
            .await
            .map_err(|e| Error::from(Box::<dyn std::error::Error + Send + Sync>::from(e)))?;

        Ok(parts.task_id.to_string())
    }

    /// Enqueue un job de restauration (interactive)
    #[instrument(skip(self, job))]
    pub async fn enqueue_restore(&mut self, job: RestoreJobData) -> Result<String, Error> {
        let parts = self
            .interactive_storage
            .push(job.clone())
            .await
            .map_err(|e| Error::from(Box::<dyn std::error::Error + Send + Sync>::from(e)))?;

        self.progress_publisher
            .create_job(
                &parts.task_id.to_string(),
                JobKind::with_restore(job.clone()),
                Some(&job.host),
            )
            .await
            .map_err(|e| Error::from(Box::<dyn std::error::Error + Send + Sync>::from(e)))?;

        Ok(parts.task_id.to_string())
    }

    /// Enqueue un job fsck (vérification de pool)
    #[instrument(skip(self))]
    pub async fn enqueue_fsck(
        &mut self,
        dry_run: bool,
        verify_chunks: bool,
    ) -> Result<String, Error> {
        let fsck_job_data = FsckJobData {
            dry_run,
            verify_chunks,
        };
        let data = MaintenanceJobData::Fsck(fsck_job_data.clone());
        let parts = self
            .maintenance_storage
            .push(data)
            .await
            .map_err(|e| Error::from(Box::<dyn std::error::Error + Send + Sync>::from(e)))?;

        self.progress_publisher
            .create_job(
                &parts.task_id.to_string(),
                JobKind::with_fsck(fsck_job_data),
                None,
            )
            .await
            .map_err(|e| Error::from(Box::<dyn std::error::Error + Send + Sync>::from(e)))?;

        Ok(parts.task_id.to_string())
    }

    /// Enqueue un job de stats (interactive queue ou maintenance selon TS). Ici maintenance.
    #[instrument(skip(self))]
    pub async fn enqueue_stats(&mut self) -> Result<String, Error> {
        let stats_job_data = StatsJobData::default();
        let data = MaintenanceJobData::Stats(stats_job_data.clone());
        let parts = self
            .maintenance_storage
            .push(data)
            .await
            .map_err(|e| Error::from(Box::<dyn std::error::Error + Send + Sync>::from(e)))?;

        self.progress_publisher
            .create_job(
                &parts.task_id.to_string(),
                JobKind::with_stats(stats_job_data),
                None,
            )
            .await
            .map_err(|e| Error::from(Box::<dyn std::error::Error + Send + Sync>::from(e)))?;

        Ok(parts.task_id.to_string())
    }

    /// Enqueue un job de cleanup_refcnt (maintenance)
    #[instrument(skip(self))]
    pub async fn enqueue_cleanup_refcnt(&mut self) -> Result<String, Error> {
        let cleanup_refcnt_job_data = CleanupRefcntJobData::default();
        let data = MaintenanceJobData::CleanupRefcnt(cleanup_refcnt_job_data.clone());
        let parts = self
            .maintenance_storage
            .push(data)
            .await
            .map_err(|e| Error::from(Box::<dyn std::error::Error + Send + Sync>::from(e)))?;

        self.progress_publisher
            .create_job(
                &parts.task_id.to_string(),
                JobKind::with_cleanup_refcnt(cleanup_refcnt_job_data),
                None,
            )
            .await
            .map_err(|e| Error::from(Box::<dyn std::error::Error + Send + Sync>::from(e)))?;

        Ok(parts.task_id.to_string())
    }
}

/// Planification Cron: utilitaires de persistance vers RedisStorage
use apalis_cron::pipe::CronPipe;
use apalis_cron::CronStream;

pub fn pipe_cron_to_backup_storage(
    schedule: apalis_cron::Schedule,
    storage: RedisStorage<ScheduleQueueJob>,
) -> CronPipe<RedisStorage<ScheduleQueueJob>> {
    CronStream::new(schedule).pipe_to_storage(storage)
}

pub fn pipe_cron_to_maintenance_storage(
    schedule: apalis_cron::Schedule,
    storage: RedisStorage<MaintenanceJobData>,
) -> CronPipe<RedisStorage<MaintenanceJobData>> {
    CronStream::new(schedule).pipe_to_storage(storage)
}

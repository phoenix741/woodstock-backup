//! Producteurs de jobs (scanner wakeup, événements, nightly)

use std::sync::Arc;

use apalis::prelude::*;
use apalis_redis::RedisStorage;
use chrono::Local;
use tracing::instrument;
use uuid::Uuid;
use woodstock::config::{BackupStatus, Backups, Hosts};
use woodstock::server::backup::retention::get_backups_to_delete;

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

    /// Applique la politique de rétention pour un hôte : calcule les sauvegardes à supprimer
    /// selon la politique configurée, et enqueue un job `Remove` pour chacune.
    ///
    /// `exclude_id` — UUID d'une sauvegarde à ne jamais supprimer (ex: backup qui vient d'être créé).
    ///
    /// Retourne le nombre de jobs de suppression enqueués.
    #[instrument(skip(self))]
    pub async fn enforce_retention_for_host(
        &mut self,
        host: &str,
        exclude_id: Option<Uuid>,
    ) -> Result<usize, Error> {
        let backups = self.backups.get_backups(host).await;

        // Re-enqueue any backup already stuck in a Removing state (interrupted
        // job whose retries were exhausted). Without this, they would remain
        // stuck indefinitely since the retention algorithm only considers
        // Completed backups.
        let stuck: Vec<Uuid> = backups
            .iter()
            .filter(|b| matches!(b.status, BackupStatus::Removing(_)))
            .map(|b| b.id)
            .collect();
        let stuck_count = enqueue_retention_removals(
            host,
            &backups,
            stuck,
            &mut self.backup_storage,
            Some(&self.progress_publisher),
        )
        .await;

        let policy = self
            .hosts
            .get_schedule(host)
            .await
            .map_err(|e| Error::from(Box::<dyn std::error::Error + Send + Sync>::from(e)))?
            .backup_to_keep;

        let Some(policy) = policy else {
            return Ok(stuck_count);
        };

        let now = Local::now();
        let mut to_delete = get_backups_to_delete(&backups, &policy, now);

        if let Some(excluded) = exclude_id {
            to_delete.retain(|&id| id != excluded);
        }

        let retention_count = enqueue_retention_removals(
            host,
            &backups,
            to_delete,
            &mut self.backup_storage,
            Some(&self.progress_publisher),
        )
        .await;

        Ok(stuck_count + retention_count)
    }
}

/// Enqueue un job `Remove` pour chaque backup de la liste `to_delete`.
///
/// Cette free function est la brique partagée entre :
/// - [`Producers::enforce_retention_for_host`] (appelée depuis une mutation GraphQL)
/// - [`crate::jobs::workers::JobExecutors::enforce_retention_policy`] (appelée post-backup)
///
/// Retourne le nombre de jobs enqueués avec succès.
pub(crate) async fn enqueue_retention_removals(
    host: &str,
    backups: &[woodstock::config::Backup],
    to_delete: Vec<Uuid>,
    storage: &mut apalis_redis::RedisStorage<BackupQueueJob>,
    progress: Option<&ProgressPublisher>,
) -> usize {
    let mut enqueued = 0usize;
    for backup_id in to_delete {
        let backup_number = backups
            .iter()
            .find(|b| b.id == backup_id)
            .map_or(0, |b| b.number);

        let remove_data = RemoveJobData {
            host: host.to_string(),
            id: backup_id,
            number: backup_number,
            ..Default::default()
        };

        match storage
            .push(BackupQueueJob::Remove(remove_data.clone()))
            .await
        {
            Ok(parts) => {
                if let Some(publi) = progress {
                    let _ = publi
                        .create_job(
                            &parts.task_id.to_string(),
                            JobKind::with_remove(remove_data),
                            Some(host),
                        )
                        .await;
                }
                enqueued += 1;
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to enqueue retention removal for {}/{}: {}",
                    host,
                    backup_id,
                    e
                );
            }
        }
    }
    enqueued
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

pub fn pipe_cron_to_nightly_storage(
    schedule: apalis_cron::Schedule,
    storage: RedisStorage<ScheduleQueueJob>,
) -> CronPipe<RedisStorage<ScheduleQueueJob>> {
    CronStream::new(schedule).pipe_to_storage(storage)
}

//! Queue service for API business logic

use std::sync::Arc;

use apalis::prelude::BackendExpose;
use apalis_redis::{connect, Config as RedisConfig, RedisStorage};
use eyre::Result;
use woodstock::config::Configuration;

use crate::jobs::types::{
    ArchiveJobData, BackupQueueJob, MaintenanceJobData, QueueName, RestoreJobData, ScheduleQueueJob,
};

/// Queue service for API business logic
/// Provides shared logic for REST and GraphQL endpoints
#[allow(dead_code)]
pub struct QueueService {
    /// Redis-backed storages (Apalis) par file
    schedule_storage: RedisStorage<ScheduleQueueJob>,
    backup_storage: RedisStorage<BackupQueueJob>,
    interactive_storage: RedisStorage<RestoreJobData>,
    maintenance_storage: RedisStorage<MaintenanceJobData>,
    archive_storage: RedisStorage<ArchiveJobData>,
}

impl QueueService {
    /// Create new QueueService instance
    pub async fn new(config: Arc<Configuration>) -> Result<Self> {
        // Utilise l'URL Redis de woodstock-rs
        let redis_url = config.redis_url();
        let conn_schedule = connect(redis_url.clone()).await?;
        let conn_backup = connect(redis_url.clone()).await?;
        let conn_interactive = connect(redis_url.clone()).await?;
        let conn_maintenance = connect(redis_url.clone()).await?;
        let conn_archive = connect(redis_url.clone()).await?;

        let schedule_storage = RedisStorage::new_with_config(
            conn_schedule,
            RedisConfig::default().set_namespace(QueueName::Schedule.as_str()),
        );
        let backup_storage = RedisStorage::new_with_config(
            conn_backup,
            RedisConfig::default().set_namespace(QueueName::Backup.as_str()),
        );
        let interactive_storage = RedisStorage::new_with_config(
            conn_interactive,
            RedisConfig::default().set_namespace(QueueName::Interactive.as_str()),
        );
        let maintenance_storage = RedisStorage::new_with_config(
            conn_maintenance,
            RedisConfig::default().set_namespace(QueueName::Maintenance.as_str()),
        );
        let archive_storage = RedisStorage::new_with_config(
            conn_archive,
            RedisConfig::default().set_namespace(QueueName::Archive.as_str()),
        );

        Ok(Self {
            schedule_storage,
            backup_storage,
            interactive_storage,
            maintenance_storage,
            archive_storage,
        })
    }

    pub fn schedule_storage(&self) -> RedisStorage<ScheduleQueueJob> {
        self.schedule_storage.clone()
    }

    pub fn backup_storage(&self) -> RedisStorage<BackupQueueJob> {
        self.backup_storage.clone()
    }

    pub fn interactive_storage(&self) -> RedisStorage<RestoreJobData> {
        self.interactive_storage.clone()
    }

    pub fn maintenance_storage(&self) -> RedisStorage<MaintenanceJobData> {
        self.maintenance_storage.clone()
    }

    pub fn archive_storage(&self) -> RedisStorage<ArchiveJobData> {
        self.archive_storage.clone()
    }

    /// Get queue statistics
    pub async fn get_queue_stats(&self) -> Result<QueueStats> {
        // Récupère les stats sur chaque file et agrège
        let s_backup = self.backup_storage.stats().await.unwrap_or_default();
        let s_inter = self.interactive_storage.stats().await.unwrap_or_default();
        let s_maint = self.maintenance_storage.stats().await.unwrap_or_default();
        let s_archive = self.archive_storage.stats().await.unwrap_or_default();

        let pending = s_backup.pending + s_inter.pending + s_maint.pending + s_archive.pending;
        let running = s_backup.running + s_inter.running + s_maint.running + s_archive.running;
        let success = s_backup.success + s_inter.success + s_maint.success + s_archive.success;
        let failed = s_backup.failed + s_inter.failed + s_maint.failed + s_archive.failed;
        let dead = s_backup.dead + s_inter.dead + s_maint.dead + s_archive.dead;

        Ok(QueueStats {
            pending,
            running,
            success,
            failed,
            dead,
        })
    }
}

/// Queue statistics
#[derive(Debug, Clone)]
pub struct QueueStats {
    pub pending: usize,
    pub running: usize,
    pub success: usize,
    pub failed: usize,
    pub dead: usize,
}

/// Job status
#[derive(Debug, Clone)]
pub enum JobStatus {
    Waiting,
    Active,
    Completed,
    Failed,
    Delayed,
}

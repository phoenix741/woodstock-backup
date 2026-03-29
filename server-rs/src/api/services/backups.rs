//! Backups service for API business logic

use crate::api::dto::Backup;
use chrono::Local;
use eyre::Result;
use std::sync::Arc;
use uuid::Uuid;
use woodstock::{
    config::{Backup as WoodstockBackup, Backups, ShareRecord},
    manifest::Manifest,
    server::job::JobUtility,
};

/// Backups service for API business logic
/// Provides shared logic for REST and GraphQL endpoints
#[derive(Clone)]
pub struct BackupsService {
    backups: Arc<Backups>,
    job_utility: Arc<JobUtility>,
}

impl BackupsService {
    /// Create new BackupsService instance
    pub fn new(backups: Arc<Backups>, job_utility: Arc<JobUtility>) -> Self {
        Self {
            backups,
            job_utility,
        }
    }

    /// Get all backups for a host
    pub async fn get_backups(&self, hostname: &str) -> Result<Vec<Backup>> {
        let woodstock_backups = self.backups.get_backups(hostname).await;
        Ok(woodstock_backups
            .into_iter()
            .map(|backup| backup.into())
            .collect())
    }

    /// Get a specific backup by hostname and UUID
    pub async fn get_backup(&self, hostname: &str, backup_id: Uuid) -> Result<Option<Backup>> {
        if let Some(backup) = self.backups.get_backup(hostname, backup_id).await {
            Ok(Some(backup.into()))
        } else {
            Ok(None)
        }
    }

    /// Get a specific backup by hostname and sequential number (display only)
    pub async fn get_backup_by_number(
        &self,
        hostname: &str,
        backup_number: usize,
    ) -> Result<Option<Backup>> {
        if let Some(backup) = self
            .backups
            .get_backup_by_number(hostname, backup_number)
            .await
        {
            Ok(Some(backup.into()))
        } else {
            Ok(None)
        }
    }

    /// Get the last backup for a host
    pub async fn get_last_backup(&self, hostname: &str) -> Result<Option<Backup>> {
        let backups = self.get_backups(hostname).await?;
        Ok(backups.into_iter().max_by_key(|b| b.start_date))
    }

    pub async fn get_time_since_last_backup(&self, hostname: &str) -> Option<chrono::Duration> {
        self.backups.get_time_since_last_backup(hostname).await
    }

    pub async fn get_time_to_next_backup(
        &self,
        hostname: &str,
    ) -> Result<Option<chrono::Duration>> {
        self.job_utility.get_time_to_next_backup(hostname).await
    }

    pub async fn get_date_to_next_backup(
        &self,
        hostname: &str,
    ) -> Result<Option<chrono::DateTime<Local>>> {
        self.job_utility.get_date_to_next_backup(hostname).await
    }

    /// Get backup destination directory
    pub fn get_backup_destination_directory(
        &self,
        hostname: &str,
        backup_id: Uuid,
    ) -> std::path::PathBuf {
        self.backups
            .get_backup_destination_directory(hostname, backup_id)
    }

    /// Get backup log directory
    pub fn get_log_directory(&self, hostname: &str, backup_id: Uuid) -> std::path::PathBuf {
        self.backups.get_log_directory(hostname, backup_id)
    }

    /// Get the manifest for a backup share
    pub fn get_manifest(&self, hostname: &str, backup_id: Uuid, share: &str) -> Manifest {
        self.backups.get_manifest(hostname, backup_id, share)
    }

    /// Get backup share paths
    pub async fn get_backup_share_paths(&self, hostname: &str, backup_id: Uuid) -> Vec<String> {
        self.backups
            .get_backup_share_paths(hostname, backup_id)
            .await
    }

    /// Get backup share records (with snapshot info)
    pub async fn get_backup_share_records(
        &self,
        hostname: &str,
        backup_id: Uuid,
    ) -> Vec<ShareRecord> {
        self.backups
            .get_backup_share_records(hostname, backup_id)
            .await
    }

    /// Get host path
    pub fn get_host_path(&self, hostname: &str) -> std::path::PathBuf {
        self.backups.get_host_path(hostname)
    }

    /// Add or replace backup metadata
    pub async fn add_or_replace_backup(
        &self,
        hostname: &str,
        backup: &WoodstockBackup,
    ) -> Result<()> {
        self.backups.add_or_replace_backup(hostname, backup).await
    }

    /// Remove backup and its directory
    pub async fn remove_backup(&self, hostname: &str, backup_id: Uuid) -> Result<WoodstockBackup> {
        self.backups.remove_backup(hostname, backup_id).await
    }

    /// Invalidates the Redis cache entry for `hostname`'s backup list.
    ///
    /// Under normal operation this happens automatically on every write via
    /// `notify()`. Call this explicitly from the admin `clear_cache` endpoint.
    pub async fn invalidate_backup_cache(&self, hostname: &str) {
        self.backups.invalidate_backup_cache(hostname).await;
    }
}

/// # Backups Management Module
///
/// This module provides the [`Backups`] struct and associated methods for managing backup metadata,
/// directories, manifests, and share paths in the Woodstock backup system. It is responsible for
/// organizing backup data on disk, tracking backup history, and supporting operations such as
/// backup creation, cloning, removal, and metadata updates.
///
/// ## Main Structure
///
/// - [`Backups`]: Central struct for managing backup directories and metadata for a given host.
///
/// ## Key Methods
///
/// - [`Backups::new`]: Create a new `Backups` manager from a configuration.
/// - [`Backups::get_backup_destination_directory`]: Get the directory for a specific backup.
/// - [`Backups::get_manifest`]: Get the manifest for a backup/share.
/// - [`Backups::get_backups`]: List all backups for a host.
/// - [`Backups::get_backup`]: Get a specific backup by number.
/// - [`Backups::add_or_replace_backup`]: Add or update a backup entry.
/// - [`Backups::remove_backup`]: Remove a backup and its directory.
/// - [`Backups::clone_backup`]: Clone backup data for incremental backups.
/// - [`Backups::get_backup_share_paths`]: List all share paths for a backup.
/// - [`Backups::add_backup_share_path`]: Add a share path to a backup.
///
/// ## Error Handling
///
/// - Most methods return `Result` and propagate I/O or serialization errors using the `eyre` crate.
/// - Methods that read or write YAML files may return errors if the file is missing, corrupted, or not writable.
/// - Removal methods may return errors if directories cannot be deleted.
///
/// ## Panics
///
/// - Methods do not panic under normal operation. Errors are returned as `Result`.
///
/// ## Thread Safety
///
/// This struct is not thread-safe by itself. If used in a concurrent context, wrap in a mutex or use only from one thread.
///
/// ## See Also
///
/// - [`Manifest`]: For file manifest operations
/// - [`Backup`]: For backup metadata
use chrono::{Duration, Local};
use eyre::Result;
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{Deserialize, Serialize};
use std::{
    io::{Error, ErrorKind},
    path::PathBuf,
    sync::Arc,
};
use tokio::fs::{copy, create_dir_all, read_to_string, remove_dir_all};
use tokio::sync::Mutex;
use tracing::error;
use uuid::Uuid;

use crate::{
    manifest::Manifest,
    utils::{
        cache::{cache_invalidate, cache_wrap},
        path::mangle,
    },
};

use super::{Backup, Configuration};

/// Redis channel on which `BackupChangedEvent` messages are published.
pub const BACKUP_CHANGED_CHANNEL: &str = "woodstock:backup:changed";

/// TTL (in seconds) for backup metadata cache entries — 24 hours.
const CACHE_TTL_SECS: u64 = 86400;

/// Returns the Redis cache key for the backup list of `hostname`.
fn backup_cache_key(hostname: &str) -> String {
    format!("woodstock:cache:backups:{hostname}")
}

/// Event published to Redis each time a backup is created, updated,
/// or deleted. Serialized/deserialized as JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupChangedEvent {
    /// Name of the host concerned by this event.
    pub hostname: String,
    /// UUID of the backup concerned by this event.
    pub backup_id: Uuid,
    /// `true` if the backup has been removed (the file no longer exists on disk).
    pub removed: bool,
}

/// Central struct for managing backup directories and metadata for a given host.
pub struct Backups {
    /// Configuration.
    config: Arc<Configuration>,
    /// Redis connection used to publish `BackupChangedEvent` notifications.
    /// `None` if notification is not enabled (e.g., when used from the CLI).
    redis_publisher: Option<Arc<Mutex<ConnectionManager>>>,
}

impl Backups {
    #[must_use]
    pub fn new(config: Arc<Configuration>) -> Self {
        Self {
            config,
            redis_publisher: None,
        }
    }

    /// Creates a `Backups` instance that publishes a [`BackupChangedEvent`] to Redis
    /// (`woodstock:backup:changed`) after each write or deletion.
    ///
    /// Use this variant from server binaries that have access to a Redis [`ConnectionManager`].
    pub fn with_redis_publisher(
        config: Arc<Configuration>,
        conn_manager: ConnectionManager,
    ) -> Self {
        Self {
            config,
            redis_publisher: Some(Arc::new(Mutex::new(conn_manager))),
        }
    }

    /// Publishes a [`BackupChangedEvent`] to Redis if a publisher is configured,
    /// and **invalidates** the backup cache for the affected host.
    /// Serialization or connection errors are traced but ignored.
    async fn notify(&self, hostname: &str, backup_id: Uuid, removed: bool) {
        if let Some(publisher) = &self.redis_publisher {
            // Invalidate cache first so readers see fresh data on the next request.
            cache_invalidate(publisher, &backup_cache_key(hostname)).await;

            let event = BackupChangedEvent {
                hostname: hostname.to_string(),
                backup_id,
                removed,
            };
            match serde_json::to_string(&event) {
                Ok(json) => {
                    let mut conn = publisher.lock().await;
                    if let Err(e) = conn.publish::<_, _, ()>(BACKUP_CHANGED_CHANNEL, json).await {
                        error!("Failed to publish BackupChangedEvent to Redis: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to serialize BackupChangedEvent: {}", e);
                }
            }
        }
    }

    /// Reads the backup list for `hostname` directly from disk, without any cache.
    async fn read_backups_from_disk(path: PathBuf) -> Vec<Backup> {
        let backups = read_to_string(path).await;
        match backups {
            Ok(backups) => {
                let backups: std::result::Result<Vec<Backup>, serde_yaml_ng::Error> =
                    serde_yaml_ng::from_str(&backups);
                match backups {
                    Ok(backups) => backups,
                    Err(e) => {
                        error!("Failed to parse backups: {e}");
                        vec![]
                    }
                }
            }
            Err(e) => {
                if e.kind() != ErrorKind::NotFound {
                    error!("Failed to read backups: {e}");
                }
                vec![]
            }
        }
    }

    #[must_use]
    pub fn get_backup_destination_directory(&self, hostname: &str, backup_id: Uuid) -> PathBuf {
        self.config
            .path
            .hosts_path
            .join(hostname)
            .join(backup_id.to_string())
    }

    #[must_use]
    pub fn get_log_directory(&self, hostname: &str, backup_id: Uuid) -> PathBuf {
        self.get_backup_destination_directory(hostname, backup_id)
    }

    #[must_use]
    pub fn get_manifest(&self, hostname: &str, backup_id: Uuid, share: &str) -> Manifest {
        let share = mangle(share);
        Manifest::new(
            &share,
            &self.get_backup_destination_directory(hostname, backup_id),
        )
    }

    #[must_use]
    pub fn get_host_path(&self, hostname: &str) -> PathBuf {
        self.config.path.hosts_path.join(hostname)
    }

    #[must_use]
    pub async fn get_manifests(&self, hostname: &str, backup_id: Uuid) -> Vec<Manifest> {
        let shares = self.get_backup_share_paths(hostname, backup_id).await;
        shares
            .iter()
            .map(|share| self.get_manifest(hostname, backup_id, share))
            .collect()
    }

    #[must_use]
    pub async fn get_backups(&self, hostname: &str) -> Vec<Backup> {
        let path = self.get_backup_file(hostname);
        if let Some(conn) = &self.redis_publisher {
            return cache_wrap(
                conn,
                &backup_cache_key(hostname),
                CACHE_TTL_SECS,
                || async move { Self::read_backups_from_disk(path).await },
            )
            .await;
        }
        Self::read_backups_from_disk(path).await
    }

    /// Explicitly invalidates the Redis cache entry for `hostname`'s backup list.
    ///
    /// Called by the `clear_cache` admin endpoint. Under normal operation,
    /// invalidation happens automatically inside [`Self::notify`] on every write.
    pub async fn invalidate_backup_cache(&self, hostname: &str) {
        if let Some(conn) = &self.redis_publisher {
            cache_invalidate(conn, &backup_cache_key(hostname)).await;
        }
    }

    /// Get a backup by its UUID (primary key).
    #[must_use]
    pub async fn get_backup(&self, hostname: &str, backup_id: Uuid) -> Option<Backup> {
        let backups = self.get_backups(hostname).await;
        backups.into_iter().find(|b| b.id == backup_id)
    }

    /// Get a backup by its sequential display number (for CLI / backward-compat).
    #[must_use]
    pub async fn get_backup_by_number(&self, hostname: &str, number: usize) -> Option<Backup> {
        let backups = self.get_backups(hostname).await;
        backups.into_iter().find(|b| b.number == number)
    }

    #[must_use]
    pub async fn get_last_backup(&self, hostname: &str) -> Option<Backup> {
        let backups = self.get_backups(hostname).await;
        // Sort by sequential number — reliable even for backups migrated with UUID v4.
        backups.into_iter().max_by_key(|b| b.number)
    }

    #[must_use]
    pub async fn get_time_since_last_backup(&self, hostname: &str) -> Option<Duration> {
        let last_backup = self.get_last_backup(hostname).await;
        last_backup.and_then(|backup| {
            backup.end_date.map(|end_date| {
                let now = Local::now();
                now - end_date
            })
        })
    }

    #[must_use]
    pub async fn get_backup_share_paths(&self, hostname: &str, backup_id: Uuid) -> Vec<String> {
        let shares = read_to_string(self.get_share_file(hostname, backup_id)).await;

        match shares {
            Ok(shares) => serde_yaml_ng::from_str(&shares).unwrap_or(vec![]),
            Err(_) => vec![],
        }
    }

    /// Adds a share path to the backup configuration for a given host and backup number.
    ///
    /// # Arguments
    /// * `hostname` - The hostname for which to add the share path.
    /// * `backup_number` - The backup number to which the share path should be added.
    /// * `share_path` - The share path to add.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the share path is successfully added.
    /// * `Err(eyre::Report)` if an error occurs during the operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the share paths cannot be serialized or written to disk.
    pub async fn add_backup_share_path(
        &self,
        hostname: &str,
        backup_id: Uuid,
        share_path: &str,
    ) -> Result<()> {
        let mut shares = self.get_backup_share_paths(hostname, backup_id).await;

        if !shares.contains(&share_path.to_string()) {
            shares.push(share_path.to_string());
        }

        let shares = serde_yaml_ng::to_string(&shares).map_err(|_| {
            Error::new(
                ErrorKind::InvalidData,
                "Failed to serialize shares to yaml string",
            )
        })?;

        let share_file = self.get_share_file(hostname, backup_id);
        tokio::fs::write(&share_file, shares).await?;

        Ok(())
    }

    #[must_use]
    pub fn get_backup_file(&self, hostname: &str) -> PathBuf {
        self.config
            .path
            .hosts_path
            .join(hostname)
            .join("backup.yml")
    }

    #[must_use]
    pub fn get_share_file(&self, hostname: &str, backup_id: Uuid) -> PathBuf {
        self.config
            .path
            .hosts_path
            .join(hostname)
            .join(backup_id.to_string())
            .join("shares.yml")
    }

    /// Clones a backup configuration and its associated files to a new destination.
    ///
    /// # Arguments
    /// * `hostname` - The hostname for which to clone the backup.
    /// * `backup_number` - The optional backup number to clone. If None, all backups are cloned.
    /// * `destination_number` - The destination backup number.
    /// * `shares` - The list of share paths to include in the clone.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the backup is successfully cloned.
    /// * `Err(eyre::Report)` if an error occurs during the operation.
    ///
    /// # Errors
    ///
    /// Returns an error if any file operation (copying, reading, or writing) fails during the cloning process.
    pub async fn clone_backup(
        &self,
        hostname: &str,
        source_id: Option<Uuid>,
        destination_id: Uuid,
        shares: &[&str],
    ) -> Result<()> {
        let destination_directory = self.get_backup_destination_directory(hostname, destination_id);

        create_dir_all(&destination_directory).await?;

        if let Some(source_id) = source_id {
            let source_directory = self.get_backup_destination_directory(hostname, source_id);

            // Copy only manifest that correspond to new shares
            for share in shares {
                let manifest = self.get_manifest(hostname, source_id, share);
                let destination_manifest = self.get_manifest(hostname, destination_id, share);

                // Copy only if exist
                if manifest.manifest_path.exists() {
                    copy(&manifest.manifest_path, &destination_manifest.manifest_path).await?;
                }
            }

            // Copy refcnt
            let refcnt = source_directory.join("REFCNT");
            if refcnt.exists() {
                copy(&refcnt, destination_directory.join("REFCNT")).await?;
            }
        }

        Ok(())
    }

    /// Adds a new backup or replaces an existing backup for a given host.
    ///
    /// # Arguments
    /// * `hostname` - The hostname for which to add or replace the backup.
    /// * `backup` - The backup configuration to add or replace.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the backup is successfully added or replaced.
    /// * `Err(eyre::Report)` if an error occurs during the operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the backup list cannot be read or written to disk.
    pub async fn add_or_replace_backup(&self, hostname: &str, backup: &Backup) -> Result<()> {
        let backups = self.get_backups(hostname).await;

        // Find the index of backup.id in backup_file if found
        let index = backups
            .iter()
            .position(|b| b.id == backup.id)
            .unwrap_or(backups.len());

        // If found replace it, else add a new one
        let mut backups = backups;
        if index < backups.len() {
            backups[index] = backup.clone();
        } else {
            backups.push(backup.clone());
        }

        // Serialize and save in the backup file
        self.save(hostname, &backups).await?;
        self.notify(hostname, backup.id, false).await;

        Ok(())
    }

    /// Updates a specific backup by applying a closure to modify it.
    ///
    /// # Arguments
    /// * `hostname` - The hostname for which to update the backup.
    /// * `backup_number` - The backup number to update.
    /// * `f` - A closure that takes a mutable reference to the backup to modify.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the backup is successfully updated.
    /// * `Err(eyre::Report)` if the backup cannot be found or saved.
    pub async fn update_backup<F>(&self, hostname: &str, backup_id: Uuid, mut f: F) -> Result<()>
    where
        F: FnMut(&mut Backup),
    {
        let backups = self.get_backups(hostname).await;

        // Find the backup to update
        let index = backups
            .iter()
            .position(|b| b.id == backup_id)
            .ok_or_else(|| eyre::eyre!("Backup {} not found for host {}", backup_id, hostname))?;

        let mut backups = backups;
        f(&mut backups[index]);

        // Save the updated backups
        self.save(hostname, &backups).await?;
        self.notify(hostname, backup_id, false).await;

        Ok(())
    }

    /// Removes a backup for a given host and backup number.
    ///
    /// # Arguments
    /// * `hostname` - The hostname for which to remove the backup.
    /// * `backup_number` - The backup number to remove.
    ///
    /// # Returns
    ///
    /// * `Ok(Backup)` if the backup is successfully removed.
    /// * `Err(eyre::Report)` if an error occurs during the operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the backup cannot be found, read, or written to disk.
    pub async fn remove_backup(&self, hostname: &str, backup_id: Uuid) -> Result<Backup> {
        let backup_destination = self.get_backup_destination_directory(hostname, backup_id);

        let mut backups = self.get_backups(hostname).await;

        // Find the index of backup.id in backup_file
        let index = backups
            .iter()
            .position(|b| b.id == backup_id)
            .ok_or_else(|| {
                Error::new(ErrorKind::NotFound, format!("Backup {backup_id} not found"))
            })?;

        // Remove the backup from the list
        let backup = backups.remove(index);

        // Serialize and save in the backup file
        self.save(hostname, &backups).await?;

        remove_dir_all(&backup_destination).await?;
        self.notify(hostname, backup_id, true).await;

        Ok(backup)
    }

    /// Saves the list of backups for a given host to disk.
    ///
    /// # Arguments
    /// * `hostname` - The hostname for which to save the backups.
    /// * `backups` - The list of backups to save.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the backups are successfully saved.
    /// * `Err(eyre::Report)` if an error occurs during the operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the backups cannot be serialized or written to disk.
    async fn save(&self, hostname: &str, backups: &Vec<Backup>) -> Result<()> {
        let backups = serde_yaml_ng::to_string(&backups).map_err(|_| {
            Error::new(
                ErrorKind::InvalidData,
                "Failed to serialize backups to yaml string",
            )
        })?;

        let backup_file = self.get_backup_file(hostname);

        tokio::fs::write(&backup_file, backups).await?;

        Ok(())
    }
}

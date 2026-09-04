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
use tokio::fs::{copy, create_dir_all, read_to_string, remove_dir_all, rename};
use tokio::sync::Mutex;
use tracing::error;
use uuid::Uuid;

use crate::{
    manifest::Manifest,
    utils::{
        cache::{cache_invalidate, cache_wrap},
        lock_redis::{FileLockOperation, LockOperation, PoolLockRedis},
        path::mangle,
    },
};

use super::{Backup, Configuration};

/// Snapshot method used to protect a share before backup.
/// Stored in YAML for human readability.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShareSnapshotMethod {
    #[default]
    None,
    Btrfs,
    Vss,
}

/// Per-share record stored in `shares.yml`, including snapshot metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShareRecord {
    pub path: String,
    #[serde(default)]
    pub snapshot_method: ShareSnapshotMethod,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_failure_reason: Option<String>,
}

/// Allows deserializing both the legacy `Vec<String>` format and the new `Vec<ShareRecord>` format.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum ShareEntry {
    Path(String),
    Record(ShareRecord),
}

impl From<ShareEntry> for ShareRecord {
    fn from(entry: ShareEntry) -> Self {
        match entry {
            ShareEntry::Path(path) => ShareRecord {
                path,
                snapshot_method: ShareSnapshotMethod::None,
                snapshot_failure_reason: None,
            },
            ShareEntry::Record(record) => record,
        }
    }
}

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

    /// Reads backups directly from disk, bypassing any cache.
    ///
    /// Must be used by **all write operations** so that the read-modify-write
    /// cycle always works on the authoritative on-disk state, not on a
    /// potentially stale cache entry.
    async fn read_backups_for_write(&self, hostname: &str) -> Result<Vec<Backup>> {
        let path = self.get_backup_file(hostname);
        Self::read_backups_from_disk(path).await
    }

    /// Acquires an exclusive Redis lock scoped to the `backup.yml` file of
    /// `hostname`.  Held for the duration of any read-modify-write cycle to
    /// prevent concurrent writes from racing on the same file.
    async fn lock_host_for_write(&self, hostname: &str) -> Result<PoolLockRedis> {
        let redis_url = self.config.redis_url();
        let backup_file = self.get_backup_file(hostname);
        let lock = PoolLockRedis::new_with_path(
            &redis_url,
            &backup_file,
            LockOperation::File(FileLockOperation::Write),
        )
        .await?
        .lock_exclusive()
        .await?;
        Ok(lock)
    }

    /// Reads the backup list for `hostname` directly from disk, without any cache.
    async fn read_backups_from_disk(path: PathBuf) -> Result<Vec<Backup>> {
        match read_to_string(&path).await {
            Ok(content) => serde_yaml_ng::from_str(&content)
                .map_err(|e| eyre::eyre!("Failed to parse backups from {path:?}: {e}")),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(vec![]),
            Err(e) => Err(eyre::eyre!("Failed to read backups from {path:?}: {e}")),
        }
    }

    /// Reads the backup list for `hostname` from disk, tolerating read/parse failures by
    /// logging and falling back to an empty list. Only safe for read-only callers
    /// (`get_backups`) — write paths must use [`Self::read_backups_from_disk`] directly
    /// and propagate the error instead, or risk overwriting `backup.yml` with data loss.
    async fn read_backups_from_disk_lenient(path: PathBuf) -> Vec<Backup> {
        match Self::read_backups_from_disk(path).await {
            Ok(backups) => backups,
            Err(e) => {
                error!("{e}");
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
                || async move { Self::read_backups_from_disk_lenient(path).await },
            )
            .await;
        }
        Self::read_backups_from_disk_lenient(path).await
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
    pub async fn get_backup_share_records(
        &self,
        hostname: &str,
        backup_id: Uuid,
    ) -> Vec<ShareRecord> {
        let shares = read_to_string(self.get_share_file(hostname, backup_id)).await;

        match shares {
            Ok(shares) => {
                let entries: Vec<ShareEntry> = serde_yaml_ng::from_str(&shares).unwrap_or_default();
                entries.into_iter().map(ShareRecord::from).collect()
            }
            Err(_) => vec![],
        }
    }

    #[must_use]
    pub async fn get_backup_share_paths(&self, hostname: &str, backup_id: Uuid) -> Vec<String> {
        self.get_backup_share_records(hostname, backup_id)
            .await
            .into_iter()
            .map(|r| r.path)
            .collect()
    }

    /// Adds a share record (with snapshot metadata) to the backup configuration for a given host and backup.
    pub async fn add_backup_share_record(
        &self,
        hostname: &str,
        backup_id: Uuid,
        record: ShareRecord,
    ) -> Result<()> {
        let mut records = self.get_backup_share_records(hostname, backup_id).await;

        // Replace existing entry for this path or add new one
        if let Some(existing) = records.iter_mut().find(|r| r.path == record.path) {
            *existing = record;
        } else {
            records.push(record);
        }

        let shares = serde_yaml_ng::to_string(&records).map_err(|_| {
            Error::new(
                ErrorKind::InvalidData,
                "Failed to serialize share records to yaml string",
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
        let _lock = self.lock_host_for_write(hostname).await?;
        let backups = self.read_backups_for_write(hostname).await?;

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
        let _lock = self.lock_host_for_write(hostname).await?;
        let backups = self.read_backups_for_write(hostname).await?;

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

        let _lock = self.lock_host_for_write(hostname).await?;
        let mut backups = self.read_backups_for_write(hostname).await?;

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

        // Invalidate the cache immediately after the yaml write, regardless of
        // what happens to the directory removal below. Cache coherence must be
        // tied to the disk write, not to the subsequent filesystem operation.
        // Failing to do this causes stale cache entries to be written back to
        // disk by the next job, resurrecting removed entries (infinite loop).
        self.notify(hostname, backup_id, true).await;

        match remove_dir_all(&backup_destination).await {
            Ok(()) => {}
            Err(e) if e.kind() == ErrorKind::NotFound => {
                // Directory already absent — treat as already removed.
            }
            Err(e) => return Err(e.into()),
        }

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
        let content = serde_yaml_ng::to_string(&backups).map_err(|_| {
            Error::new(
                ErrorKind::InvalidData,
                "Failed to serialize backups to yaml string",
            )
        })?;

        let backup_file = self.get_backup_file(hostname);

        // Write-temp-then-rename instead of a direct write: `tokio::fs::write` is
        // open+truncate+write, not crash-safe — a kill mid-write leaves `backup.yml`
        // truncated/corrupt. `rename` within the same directory is atomic, so a crash
        // here either leaves the old file untouched or the new one fully in place.
        let tmp_file = backup_file.with_extension("yml.tmp");
        tokio::fs::write(&tmp_file, content).await?;
        rename(&tmp_file, &backup_file).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BackupStatus, Configuration};
    use chrono::Local;

    fn sample_backup(id: Uuid, number: usize) -> Backup {
        Backup {
            id,
            number,
            status: BackupStatus::InProgress,
            start_date: Local::now(),
            end_date: None,
            error_count: 0,
            error_message: None,
            file_count: 0,
            new_file_count: 0,
            removed_file_count: 0,
            modified_file_count: 0,
            existing_file_count: 0,
            file_size: 0,
            new_file_size: 0,
            modified_file_size: 0,
            existing_file_size: 0,
            compressed_file_size: 0,
            new_compressed_file_size: 0,
            modified_compressed_file_size: 0,
            existing_compressed_file_size: 0,
            speed: 0.0,
            agent_version: None,
        }
    }

    #[tokio::test]
    async fn read_backups_from_disk_missing_file_is_empty_ok() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backup.yml");

        let result = Backups::read_backups_from_disk(path).await;

        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn read_backups_from_disk_corrupt_file_is_err() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backup.yml");
        tokio::fs::write(&path, b"{ this is not valid yaml for a Vec<Backup>")
            .await
            .unwrap();

        let result = Backups::read_backups_from_disk(path).await;

        // Must fail loudly rather than silently returning an empty list — a caller in
        // the write path would otherwise overwrite backup.yml with a near-empty list,
        // wiping every previously tracked backup for that host.
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn save_round_trip_via_add_or_replace_backup() {
        let dir = tempfile::tempdir().unwrap();
        let config = Arc::new(Configuration::from_backup_path(dir.path().to_path_buf()));
        create_dir_all(config.path.hosts_path.join("myhost"))
            .await
            .unwrap();
        let backups = Backups::new(config);

        let id = Uuid::now_v7();
        backups
            .add_or_replace_backup("myhost", &sample_backup(id, 1))
            .await
            .unwrap();

        let loaded = backups.get_backups("myhost").await;
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, id);

        // The atomic write must not leave a stray .yml.tmp file behind.
        let tmp_file = backups.get_backup_file("myhost").with_extension("yml.tmp");
        assert!(!tmp_file.exists());

        // A second write (replace, not append) must still round-trip cleanly.
        let mut updated = sample_backup(id, 1);
        updated.status = BackupStatus::Completed;
        backups
            .add_or_replace_backup("myhost", &updated)
            .await
            .unwrap();

        let loaded = backups.get_backups("myhost").await;
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].status, BackupStatus::Completed);
    }
}

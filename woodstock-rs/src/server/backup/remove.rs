use std::sync::Arc;

use chrono::Local;
use eyre::Result;
use tracing::info;
use uuid::Uuid;

use crate::{
    config::{BackupStatus, Backups, Configuration, Context},
    events::create_event_backup_remove,
    pool::{PoolManager, Refcnt, RefcntApplySens},
    utils::compression::CompressionFormat,
    ChunkAlgorithm, EventSource,
};

pub struct BackupRemove {
    /// The hostname associated with the backup.
    hostname: String,
    /// The UUID v7 identifier of the current backup.
    backup_id: Uuid,
    /// The source of events for the backup removal process.
    source: EventSource,
    /// The chunk algorithm used for hash processing.
    algorithm: ChunkAlgorithm,
    /// The compression format used for the backup.
    compression_format: CompressionFormat,

    /// The configuration for the backup removal process.
    config: Arc<Configuration>,
    /// The backups configuration.
    backups: Arc<Backups>,
}

impl BackupRemove {
    /// Creates a new instance of `BackupRemove`.
    ///
    /// # Arguments
    /// * `hostname` - The hostname of the backup to remove.
    /// * `backup_id` - The UUID v7 identifier of the backup to remove.
    /// * `backup_number` - The sequential display number of the backup.
    /// * `ctxt` - The context containing the event source.
    /// * `config` - The configuration for the backup system.
    ///
    /// # Returns
    ///
    /// A new instance of `BackupRemove`.
    #[must_use]
    pub fn new(
        hostname: &str,
        backup_id: Uuid,
        ctxt: &Context,

        config: Arc<Configuration>,
        backups: Arc<Backups>,
    ) -> Self {
        let destination_directory = backups.get_backup_destination_directory(hostname, backup_id);

        info!("Initialize backup remover for {hostname}/{backup_id} in {destination_directory:?}");

        BackupRemove {
            hostname: hostname.to_string(),
            backup_id,
            source: ctxt.source,
            algorithm: config.chunk_algorithm,
            compression_format: config.compression_format,
            config,
            backups,
        }
    }

    /// Copy the references count from the backup to the pool.
    ///
    ///# Returns
    ///
    /// * `Ok(())` if the copy operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the copy operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the copy operation fails.
    pub async fn add_refcnt_to_pool(&self) -> Result<()> {
        info!("Add references count to pool");

        let host_refcnt_file = self
            .backups
            .get_backup_destination_directory(&self.hostname, self.backup_id);

        PoolManager::new(self.config.clone())
            .remove_refcnt(host_refcnt_file, &self.hostname, self.backup_id)
            .await?;

        Ok(())
    }

    /// Removes reference counts for the host.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the removal process.
    ///
    /// # Errors
    ///
    /// Returns an error if the reference count removal fails.
    pub async fn remove_refcnt_of_host(&self) -> Result<()> {
        let from_directory = self
            .backups
            .get_backup_destination_directory(&self.hostname, self.backup_id);

        let host_directory = self.backups.get_host_path(&self.hostname);

        let mut backup_refcnt = Refcnt::new(&from_directory);
        backup_refcnt.load_refcnt(false).await;

        Refcnt::apply_all_from(
            &host_directory,
            &backup_refcnt,
            &RefcntApplySens::Decrease,
            &Local::now(),
            self.algorithm,
            self.compression_format,
        )
        .await?;

        Ok(())
    }

    /// Removes the backup.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the removal process.
    ///
    /// # Errors
    ///
    /// Returns an error if the backup removal fails.
    pub async fn remove_backup(&self) -> Result<()> {
        // Fetch number before removal (backup.yml will be deleted during remove)
        let num = self
            .backups
            .get_backup(&self.hostname, self.backup_id)
            .await
            .map(|b| b.number)
            .unwrap_or(0);

        let shares = self
            .backups
            .get_backup_share_paths(&self.hostname, self.backup_id)
            .await;
        let shares = shares
            .iter()
            .map(std::string::String::as_str)
            .collect::<Vec<&str>>();

        self.backups
            .remove_backup(&self.hostname, self.backup_id)
            .await?;

        create_event_backup_remove(
            &self.config,
            &self.config.path.events_path,
            self.source,
            &self.hostname,
            self.backup_id,
            num,
            &shares,
        )
        .await?;

        Ok(())
    }

    /// Saves the backup with the specified status.
    ///
    /// # Arguments
    /// * `status` - The status of the backup.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the save operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the save operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the save operation fails.
    pub async fn save_backup(&self, status: BackupStatus) -> Result<()> {
        info!("Save backup removal status (status = {status:?})");

        let mut backup = self
            .backups
            .get_backup(&self.hostname, self.backup_id)
            .await
            .ok_or_else(|| eyre::eyre!("Backup {}/{} not found", self.hostname, self.backup_id))?;
        backup.status = status;

        self.backups
            .add_or_replace_backup(&self.hostname, &backup)
            .await?;

        Ok(())
    }
}

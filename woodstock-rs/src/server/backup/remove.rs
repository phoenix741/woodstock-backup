use std::sync::Arc;

use eyre::Result;
use tracing::info;
use uuid::Uuid;

use crate::{
    config::{Backups, Configuration, Context},
    events::create_event_backup_remove,
    pool::PoolManager,
    EventSource,
};

pub struct BackupRemove {
    /// The hostname associated with the backup.
    hostname: String,
    /// The UUID v7 identifier of the current backup.
    backup_id: Uuid,
    /// The source of events for the backup removal process.
    source: EventSource,

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
            config,
            backups,
        }
    }

    /// Finalizes the Pool V3 removal publication for this backup.
    ///
    ///# Returns
    ///
    /// * `Ok(())` if the copy operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the copy operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the copy operation fails.
    pub async fn finalize_pool_removal(&self) -> Result<()> {
        info!("Finalize pool v3 backup removal");

        PoolManager::new(self.config.clone())
            .finalize_backup_removal(&self.hostname, self.backup_id)
            .await?;

        Ok(())
    }

    /// Cleans up the host-side removal bookkeeping for this backup.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during the removal process.
    ///
    /// # Errors
    ///
    /// Returns an error if the reference count removal fails.
    pub async fn cleanup_host_removal_state(&self) -> Result<()> {
        info!(
            "Pool V3 removal does not update host REFCNT files for backup {}/{}",
            self.hostname, self.backup_id
        );
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
}

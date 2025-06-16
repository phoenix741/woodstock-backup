use eyre::Result;
use log::info;
use std::time::SystemTime;

use crate::{
    config::{Backups, Configuration, Context},
    events::create_event_backup_remove,
    pool::{remove_refcnt_to_pool, Refcnt, RefcntApplySens},
    EventSource,
};

pub struct BackupRemove {
    /// The hostname associated with the backup.
    hostname: String,
    /// The ID of the current backup.
    current_backup_id: usize,
    /// The source of events for the backup removal process.
    source: EventSource,
    /// The configuration for the backup removal process.
    config: Configuration,
}

impl BackupRemove {
    /// Creates a new instance of `BackupRemove`.
    ///
    /// # Arguments
    /// * `hostname` - The hostname of the backup to remove.
    /// * `backup_number` - The backup number to remove.
    /// * `ctxt` - The context containing the event source.
    /// * `config` - The configuration for the backup system.
    ///
    /// # Returns
    ///
    /// A new instance of `BackupRemove`.
    #[must_use]
    pub fn new(
        hostname: &str,
        backup_number: usize,
        ctxt: &Context,
        config: &Configuration,
    ) -> Self {
        let backups = Backups::new(config);
        let destination_directory =
            backups.get_backup_destination_directory(hostname, backup_number);

        info!(
            "Initialize backup remover for {hostname}/{backup_number} in {destination_directory:?}"
        );

        BackupRemove {
            hostname: hostname.to_string(),
            current_backup_id: backup_number,
            source: ctxt.source,
            config: config.clone(),
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

        let backups = Backups::new(&self.config);
        let host_refcnt_file =
            backups.get_backup_destination_directory(&self.hostname, self.current_backup_id);

        remove_refcnt_to_pool(
            &self.config,
            host_refcnt_file,
            &self.hostname,
            self.current_backup_id,
        )
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
        let backups = Backups::new(&self.config);
        let from_directory =
            backups.get_backup_destination_directory(&self.hostname, self.current_backup_id);

        let host_directory = backups.get_host_path(&self.hostname);

        let mut backup_refcnt = Refcnt::new(&from_directory);
        backup_refcnt.load_refcnt(false).await;

        Refcnt::apply_all_from(
            &host_directory,
            &backup_refcnt,
            &RefcntApplySens::Decrease,
            &SystemTime::now(),
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
        let backups = Backups::new(&self.config);
        backups
            .remove_backup(&self.hostname, self.current_backup_id)
            .await?;

        let shares = backups
            .get_backup_share_paths(&self.hostname, self.current_backup_id)
            .await;
        let shares = shares
            .iter()
            .map(std::string::String::as_str)
            .collect::<Vec<&str>>();

        create_event_backup_remove(
            &self.config.path.events_path,
            self.source,
            &self.hostname,
            self.current_backup_id,
            &shares,
        )
        .await?;

        Ok(())
    }
}

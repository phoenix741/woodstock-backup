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
use eyre::Result;
use log::error;
use std::{
    io::{Error, ErrorKind},
    path::PathBuf,
};
use tokio::fs::{copy, create_dir_all, read_to_string, remove_dir_all};

use crate::{manifest::Manifest, utils::path::mangle};

use super::{Backup, Configuration};

/// Central struct for managing backup directories and metadata for a given host.
pub struct Backups {
    /// Path to the directory containing host backup data.
    config_host_path: PathBuf,
}

impl Backups {
    #[must_use]
    pub fn new(config: &Configuration) -> Self {
        Self {
            config_host_path: config.path.hosts_path.clone(),
        }
    }

    #[must_use]
    pub fn get_backup_destination_directory(
        &self,
        hostname: &str,
        backup_number: usize,
    ) -> PathBuf {
        self.config_host_path
            .join(hostname)
            .join(backup_number.to_string())
    }

    #[must_use]
    pub fn get_log_directory(&self, hostname: &str, backup_number: usize) -> PathBuf {
        self.get_backup_destination_directory(hostname, backup_number)
    }

    #[must_use]
    pub fn get_manifest(&self, hostname: &str, backup_number: usize, share: &str) -> Manifest {
        let share = mangle(share);
        Manifest::new(
            &share,
            &self.get_backup_destination_directory(hostname, backup_number),
        )
    }

    #[must_use]
    pub fn get_host_path(&self, hostname: &str) -> PathBuf {
        self.config_host_path.join(hostname)
    }

    #[must_use]
    pub async fn get_manifests(&self, hostname: &str, backup_number: usize) -> Vec<Manifest> {
        let shares = self.get_backup_share_paths(hostname, backup_number).await;
        shares
            .iter()
            .map(|share| self.get_manifest(hostname, backup_number, share))
            .collect()
    }

    #[must_use]
    pub async fn get_backups(&self, hostname: &str) -> Vec<Backup> {
        let backups = read_to_string(self.get_backup_file(hostname)).await;

        match backups {
            Ok(backups) => {
                let backups: std::result::Result<Vec<Backup>, serde_yaml::Error> =
                    serde_yaml::from_str(&backups);
                match backups {
                    Ok(backups) => backups,
                    Err(e) => {
                        error!("Failed to parse backups: {e}");
                        vec![]
                    }
                }
            }
            Err(_) => vec![],
        }
    }

    #[must_use]
    pub async fn get_backup(&self, hostname: &str, backup_number: usize) -> Option<Backup> {
        let backups = self.get_backups(hostname).await;
        let backup = backups
            .iter()
            .find(|&backup| backup.number == backup_number);

        backup.cloned()
    }

    #[must_use]
    pub async fn get_last_backup(&self, hostname: &str) -> Option<Backup> {
        let backups = self.get_backups(hostname).await;
        let backup = backups.iter().max_by_key(|backup| backup.number);

        backup.cloned()
    }

    #[must_use]
    pub async fn get_previous_backup(
        &self,
        hostname: &str,
        backup_number: usize,
    ) -> Option<Backup> {
        let backups = self.get_backups(hostname).await;
        let backup = backups
            .iter()
            .filter(|backup| backup.number < backup_number)
            .max_by_key(|backup| backup.number);

        backup.cloned()
    }

    #[must_use]
    pub async fn get_backup_share_paths(
        &self,
        hostname: &str,
        backup_number: usize,
    ) -> Vec<String> {
        let shares = read_to_string(self.get_share_file(hostname, backup_number)).await;

        match shares {
            Ok(shares) => serde_yaml::from_str(&shares).unwrap_or(vec![]),
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
        backup_number: usize,
        share_path: &str,
    ) -> Result<()> {
        let mut shares = self.get_backup_share_paths(hostname, backup_number).await;

        if !shares.contains(&share_path.to_string()) {
            shares.push(share_path.to_string());
        }

        let shares = serde_yaml::to_string(&shares).map_err(|_| {
            Error::new(
                ErrorKind::InvalidData,
                "Failed to serialize shares to yaml string",
            )
        })?;

        let share_file = self.get_share_file(hostname, backup_number);
        tokio::fs::write(&share_file, shares).await?;

        Ok(())
    }

    #[must_use]
    pub fn get_backup_file(&self, hostname: &str) -> PathBuf {
        self.config_host_path.join(hostname).join("backup.yml")
    }

    #[must_use]
    pub fn get_share_file(&self, hostname: &str, backup_number: usize) -> PathBuf {
        self.config_host_path
            .join(hostname)
            .join(backup_number.to_string())
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
        backup_number: Option<usize>,
        destination_number: usize,
        shares: &[&str],
    ) -> Result<()> {
        let destination_directory =
            self.get_backup_destination_directory(hostname, destination_number);

        create_dir_all(&destination_directory).await?;

        if let Some(backup_number) = backup_number {
            let source_directory = self.get_backup_destination_directory(hostname, backup_number);

            // Copy only manifest that correspond to new shares
            for share in shares {
                let manifest = self.get_manifest(hostname, backup_number, share);
                let destination_manifest = self.get_manifest(hostname, destination_number, share);

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

        // Find the index of backup.number in backup_file if found
        let index = backups
            .iter()
            .position(|b| b.number == backup.number)
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
    pub async fn remove_backup(&self, hostname: &str, backup_number: usize) -> Result<Backup> {
        let backup_destination = self.get_backup_destination_directory(hostname, backup_number);

        let mut backups = self.get_backups(hostname).await;

        // Find the index of backup.number in backup_file if found
        let index = backups
            .iter()
            .position(|b| b.number == backup_number)
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::NotFound,
                    format!("Backup number {backup_number} not found"),
                )
            })?;

        // Remove the backup from the list
        let backup = backups.remove(index);

        // Serialize and save in the backup file
        self.save(hostname, &backups).await?;

        remove_dir_all(&backup_destination).await?;

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
        let backups = serde_yaml::to_string(&backups).map_err(|_| {
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

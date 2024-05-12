use std::{
    io::{Error, ErrorKind},
    path::PathBuf,
};

use eyre::Result;
use tokio::fs::{copy, create_dir_all, read_to_string, remove_dir_all};

use crate::{manifest::Manifest, utils::path::mangle};

use super::{Backup, Context};

pub struct Backups {
    config_host_path: PathBuf,
}

impl Backups {
    #[must_use]
    pub fn new(ctxt: &Context) -> Self {
        Self {
            config_host_path: ctxt.config.path.hosts_path.clone(),
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
            Ok(backups) => serde_yaml::from_str(&backups).unwrap_or(vec![]),
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

                copy(&manifest.manifest_path, &destination_manifest.manifest_path).await?;
            }

            // Copy refcnt
            copy(
                source_directory.join("REFCNT"),
                destination_directory.join("REFCNT"),
            )
            .await?;
        }

        Ok(())
    }

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

//! Hosts service for API business logic

use crate::api::dto::{HostConfiguration, HostInformation};
use crate::api::services::BackupsService;
use eyre::Result;
use std::sync::Arc;

/// Hosts service for managing host configurations
#[derive(Clone)]
pub struct HostsService {
    /// woodstock-rs configuration for direct API access
    hosts: Arc<woodstock::config::Hosts>,
    /// backups service for last backup information
    backups_service: Arc<BackupsService>,
}

impl HostsService {
    /// Create new HostsService instance
    pub fn new(hosts: Arc<woodstock::config::Hosts>, backups_service: Arc<BackupsService>) -> Self {
        Self {
            hosts,
            backups_service,
        }
    }

    /// List all available hosts
    pub async fn list_hosts(&self) -> Result<Vec<String>> {
        self.hosts.list_hosts().await
    }

    /// Get host configuration by name
    pub async fn get_public_host_configuration(&self, hostname: &str) -> Result<HostConfiguration> {
        let woodstock_config = self.hosts.get_host(hostname).await?;
        Ok(woodstock_config.into())
    }

    pub async fn get_private_host_configuration(
        &self,
        hostname: &str,
    ) -> Result<woodstock::config::HostConfiguration> {
        let woodstock_config = self.hosts.get_host(hostname).await?;
        Ok(woodstock_config)
    }

    /// Get host information for API responses (list format)
    pub async fn get_host_information(&self, hostname: &str) -> Result<HostInformation> {
        // Get last backup from BackupsService
        let last_backup = self.backups_service.get_last_backup(hostname).await?;

        Ok(HostInformation {
            name: hostname.to_string(),
            last_backup,
        })
    }

    /// Clears all host and backup caches.
    ///
    /// Invalidates:
    /// - `woodstock:cache:hosts` (host list)
    /// - `woodstock:cache:host:{hostname}` for every known host
    /// - `woodstock:cache:backups:{hostname}` for every known host
    ///
    /// The host list is read fresh from disk after its own cache is cleared,
    /// so every per-host entry is always covered.
    pub async fn clear_all_caches(&self) {
        self.hosts.invalidate_hosts_list_cache().await;
        let hostnames = self.list_hosts().await.unwrap_or_default();
        for hostname in &hostnames {
            self.hosts.invalidate_host_config_cache(hostname).await;
            self.backups_service.invalidate_backup_cache(hostname).await;
        }
    }
}

//! Files service for API business logic

use crate::api::{dto::FileDescription, services::BackupsService};
use eyre::{eyre, Result};
use lru::LruCache;
use std::{num::NonZeroUsize, sync::Arc};
use tokio::sync::Mutex;
use tracing::debug;
use uuid::Uuid;
use woodstock::{
    config::{Backups, Configuration, Hosts},
    utils::path::{unmangle, vec_to_path},
    view::WoodstockView,
};

/// Files service for API business logic
/// Provides shared logic for REST and GraphQL endpoints
#[derive(Clone)]
pub struct FilesService {
    config: Arc<Configuration>,
    hosts: Arc<Hosts>,
    backups: Arc<Backups>,
    /// Backups service for managing backup operations
    backups_service: Arc<BackupsService>,

    view: Arc<Mutex<LruCache<String, Arc<Mutex<WoodstockView>>>>>,
}

impl FilesService {
    /// Create new FilesService instance
    pub fn new(
        config: Arc<Configuration>,
        hosts: Arc<Hosts>,
        backups: Arc<Backups>,
        backups_service: Arc<BackupsService>,
    ) -> Self {
        Self {
            hosts,
            backups,
            backups_service,
            view: Arc::new(Mutex::new(LruCache::new(
                // SAFETY: cache_size is validated at config load time to be ≥ 1
                NonZeroUsize::new(config.cache_size.max(1)).unwrap(),
            ))),
            config,
        }
    }

    async fn get_viewer(&self, hostname: &str, backup_id: Uuid) -> Arc<Mutex<WoodstockView>> {
        let key = format!("{}-{}", hostname, backup_id);
        let mut view_cache = self.view.lock().await;
        if let Some(view) = view_cache.get(&key) {
            return view.clone();
        }
        let view = WoodstockView::new(
            self.config.clone(),
            self.hosts.clone(),
            self.backups.clone(),
        );
        view_cache.put(key.clone(), Arc::new(Mutex::new(view)));
        // SAFETY: key was just inserted above with view_cache.put()
        view_cache.get(&key).unwrap().clone()
    }

    /// List shares for a backup
    pub async fn list_shares(
        &mut self,
        hostname: &str,
        backup_id: Uuid,
    ) -> Result<Vec<FileDescription>> {
        let Some(backup) = self.backups_service.get_backup(hostname, backup_id).await? else {
            return Err(eyre!("Backup not found"));
        };

        let start_date = backup.start_date.timestamp();
        let shares = self
            .backups_service
            .get_backup_share_paths(hostname, backup_id)
            .await;

        Ok(shares
            .into_iter()
            .map(|share| {
                let mut share: FileDescription = share.into();
                if let Some(ref mut stats) = share.stats {
                    stats.last_read = start_date;
                    stats.last_modified = start_date;
                    stats.created = start_date;
                }
                share
            })
            .collect())
    }

    /// List files in a share/path
    pub async fn list_files(
        &mut self,
        hostname: &str,
        backup_id: Uuid,
        share: &str,
        path: &[u8],
    ) -> Result<Vec<FileDescription>> {
        let share = unmangle(share);

        let path = vec_to_path(path);
        let path = path.strip_prefix("/").unwrap_or(&path).to_path_buf();

        debug!(
            "Listing files for {} backup: {}, share_path: {:?}, path: {:?}",
            hostname, backup_id, share, path
        );

        let view = self.get_viewer(hostname, backup_id).await;
        let mut view = view.lock().await;
        let entries = view
            .list_file_from_dir(&hostname, backup_id, &share, &path)
            .await?;

        let entries = entries.iter().cloned().map(Into::into).collect();

        Ok(entries)
    }
}

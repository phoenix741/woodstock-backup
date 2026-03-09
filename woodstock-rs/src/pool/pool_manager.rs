//! [`PoolManager`]: high-level owner of all pool operations.
//!
//! All logic previously spread across free functions in `pool/mod.rs` now lives
//! here. Construct once with an [`Arc<Configuration>`] and call methods.

use chrono::{DateTime, Local};
use eyre::{Result, WrapErr};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::{create_dir_all, metadata, read_dir, remove_file};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::config::Configuration;
use crate::utils::files::copy_file;

use super::{Refcnt, RefcntApplySens};

/// High-level manager for all pool operations.
///
/// # Example
///
/// ```rust,no_run
/// # use std::sync::Arc;
/// # use woodstock::pool::PoolManager;
/// # use woodstock::config::Configuration;
/// # use chrono::Local;
/// # async fn example(config: Arc<Configuration>) -> eyre::Result<()> {
/// let pool = PoolManager::new(config);
/// pool.apply_pending(&Local::now()).await?;
/// # Ok(())
/// # }
/// ```
pub struct PoolManager {
    config: Arc<Configuration>,
}

impl PoolManager {
    /// Create a new [`PoolManager`] for the given configuration.
    #[must_use]
    pub fn new(config: Arc<Configuration>) -> Self {
        Self { config }
    }

    /// Stage a backup's reference counts for addition to the pool.
    ///
    /// Copies the backup's `REFCNT` into the pending directory; the actual
    /// pool `REFCNT` is updated later by [`Self::apply_pending`].
    ///
    /// # Errors
    ///
    /// Returns an error if the source file cannot be read or the pending
    /// directory cannot be created.
    pub async fn add_refcnt<P: AsRef<Path>>(
        &self,
        filename: P,
        hostname: &str,
        backup_id: Uuid,
    ) -> Result<()> {
        let pool_refcnt_pending_path = &self.config.path.pool_refcnt_pending_path;
        create_dir_all(pool_refcnt_pending_path).await?;

        let source_path = filename.as_ref().join("REFCNT");
        let destination_path =
            pool_refcnt_pending_path.join(format!("{}_{}.add", hostname, backup_id));

        if destination_path.exists() {
            info!(
                "Refcnt file already exists in pending pool for {}, skipping copy (idempotent)",
                source_path.display()
            );
            return Ok(());
        }

        info!("Add refcnt to pending pool for {}", source_path.display());
        copy_file(source_path, destination_path).await?;
        info!("Refcnt added to pending pool");
        Ok(())
    }

    /// Stage a backup's reference counts for removal from the pool.
    ///
    /// Copies the backup's `REFCNT` into the pending directory; the actual
    /// pool `REFCNT` is updated later by [`Self::apply_pending`].
    ///
    /// # Errors
    ///
    /// Returns an error if the source file cannot be read or the pending
    /// directory cannot be created.
    pub async fn remove_refcnt<P: AsRef<Path>>(
        &self,
        filename: P,
        hostname: &str,
        backup_id: Uuid,
    ) -> Result<()> {
        let pool_refcnt_pending_path = &self.config.path.pool_refcnt_pending_path;
        create_dir_all(pool_refcnt_pending_path).await?;

        let source_path = filename.as_ref().join("REFCNT");
        let destination_path =
            pool_refcnt_pending_path.join(format!("{}_{}.remove", hostname, backup_id));

        if destination_path.exists() {
            info!(
                "Refcnt file already exists in pending pool for removal for {}, skipping copy (idempotent)",
                source_path.display()
            );
            return Ok(());
        }

        info!(
            "Remove refcnt from pending pool for {}",
            source_path.display()
        );
        copy_file(source_path, destination_path).await?;
        info!("Refcnt added to pending pool for removal");
        Ok(())
    }

    /// Apply all pending `.add` / `.remove` operations to the global `REFCNT`.
    ///
    /// **The caller must hold an exclusive pool lock before calling this.**
    ///
    /// # Errors
    ///
    /// Returns an error if the pool is dirty, file I/O fails, or the `REFCNT`
    /// cannot be saved atomically.
    pub async fn apply_pending(&self, date: &DateTime<Local>) -> Result<()> {
        info!("Applying pending refcnt operations");

        let pool_directory = &self.config.path.pool_path;
        let pending_path = &self.config.path.pool_refcnt_pending_path;
        let dirty_file_path = &self.config.path.pool_refcnt_dirty_file;

        self.assert_clean("apply_pending_refcnt_operations").await?;
        create_dir_all(pending_path).await?;

        let mut pool_refcnt = Refcnt::load_refcnt_from_path(pool_directory).await?;

        let mut add_files = Vec::new();
        let mut remove_files = Vec::new();

        if let Ok(mut dir) = read_dir(pending_path).await {
            while let Some(entry) = dir.next_entry().await? {
                let path = entry.path();
                let Some(extension) = path.extension().and_then(|s| s.to_str()) else {
                    continue;
                };
                if extension == "add" {
                    add_files.push(path);
                } else if extension == "remove" {
                    remove_files.push(path);
                }
            }
        }

        if add_files.is_empty() && remove_files.is_empty() {
            debug!("No pending refcnt operations found");
            return Ok(());
        }

        info!(
            "Processing {} add and {} remove operations",
            add_files.len(),
            remove_files.len()
        );

        let all_files: Vec<PathBuf> = add_files
            .iter()
            .chain(remove_files.iter())
            .cloned()
            .collect();

        // Warn on suspicious timestamps
        Self::check_pending_timestamps(&pool_directory.join("REFCNT"), &all_files).await;

        for path in &add_files {
            info!("Applying add refcnt operation from file: {path:?}");
            let backup_refcnt = Refcnt::load_refcnt_from_file(path).await?;
            pool_refcnt.apply_all(&backup_refcnt, &RefcntApplySens::Increase);
        }
        for path in &remove_files {
            info!("Applying remove refcnt operation from file: {path:?}");
            let backup_refcnt = Refcnt::load_refcnt_from_file(path).await?;
            pool_refcnt.apply_all(&backup_refcnt, &RefcntApplySens::Decrease);
        }

        // Write dirty marker BEFORE saving REFCNT (crash-safe two-phase commit)
        let file_names: Vec<String> = all_files
            .iter()
            .map(|p| {
                p.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        Self::write_dirty_marker(dirty_file_path, file_names).await?;

        pool_refcnt
            .repair(pool_directory, self.config.chunk_algorithm)
            .await?;
        pool_refcnt
            .save_refcnt(date, true, self.config.compression_format)
            .await?;

        debug!("REFCNT saved atomically");

        for file in add_files.iter().chain(remove_files.iter()) {
            if let Err(e) = remove_file(file).await {
                error!(
                    "Failed to remove pending file {:?}: {}",
                    file.file_name().unwrap_or_default(),
                    e
                );
            } else {
                debug!(
                    "Removed pending file: {:?}",
                    file.file_name().unwrap_or_default()
                );
            }
        }

        if let Err(e) = remove_file(dirty_file_path).await {
            warn!(
                "Failed to remove dirty file - pool may appear dirty on next check: {}",
                e
            );
        } else {
            debug!("Dirty file removed - pool is clean");
        }

        info!("All pending refcnt operations applied successfully");
        Ok(())
    }

    /// Clean up a dirty pool state by removing all pending files and the
    /// dirty-marker file.
    ///
    /// Only call this after a confirmed crash recovery via `fsck --repair`.
    ///
    /// # Errors
    ///
    /// Returns an error if file removal fails.
    pub async fn cleanup_dirty(&self) -> Result<()> {
        info!("Cleaning up dirty pool state");

        let pending_path = &self.config.path.pool_refcnt_pending_path;
        let dirty_file = &self.config.path.pool_refcnt_dirty_file;

        let mut deleted_count = 0;
        if let Ok(mut dir) = read_dir(pending_path).await {
            while let Some(entry) = dir.next_entry().await? {
                let path = entry.path();
                if path.is_file() {
                    if let Err(e) = remove_file(&path).await {
                        error!("Failed to delete pending file {:?}: {}", path, e);
                    } else {
                        deleted_count += 1;
                        debug!(
                            "Deleted pending file: {:?}",
                            path.file_name().unwrap_or_default()
                        );
                    }
                }
            }
        }
        info!("Deleted {} pending files", deleted_count);

        if let Err(e) = remove_file(dirty_file).await {
            if e.kind() != std::io::ErrorKind::NotFound {
                error!("Failed to remove dirty file {:?}: {}", dirty_file, e);
                return Err(eyre::eyre!("Failed to remove dirty marker file"));
            }
            debug!(
                "Dirty marker file {:?} already absent — pool is clean",
                dirty_file
            );
        }

        info!("Dirty pool state cleaned up successfully");
        Ok(())
    }

    /// Count the number of pending `.add` / `.remove` operation files.
    ///
    /// # Errors
    ///
    /// Returns an error if the pending directory cannot be read.
    pub async fn count_pending(&self) -> Result<usize> {
        let pending_path = &self.config.path.pool_refcnt_pending_path;

        if !pending_path.exists() {
            return Ok(0);
        }

        let mut count = 0;
        let mut entries = read_dir(pending_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "add" || ext == "remove" {
                        count += 1;
                    }
                }
            }
        }
        Ok(count)
    }

    /// Check whether the pool is currently in a dirty (crashed) state.
    ///
    /// Returns `Some(path)` if the dirty-marker file exists, `None` if clean.
    ///
    /// # Errors
    ///
    /// Returns an error if the file-system check fails.
    pub async fn is_dirty(&self) -> Result<Option<PathBuf>> {
        let dirty_file = &self.config.path.pool_refcnt_dirty_file;
        if dirty_file.exists() {
            Ok(Some(dirty_file.to_path_buf()))
        } else {
            Ok(None)
        }
    }

    /// Assert that the pool is clean, returning a descriptive error if dirty.
    ///
    /// # Arguments
    ///
    /// * `operation` — Name of the planned operation (used in the error message).
    ///
    /// # Errors
    ///
    /// Returns an error if the pool is dirty or if the file-system check fails.
    pub async fn assert_clean(&self, operation: &str) -> Result<()> {
        if let Some(dirty_file) = self.is_dirty().await? {
            if let Ok(content) = tokio::fs::read_to_string(&dirty_file).await {
                if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&content) {
                    error!(
                        "Pool is in DIRTY state - {} cannot proceed. Dirty file created at {}, processing {} files. Run 'fsck --repair' to recover.",
                        operation,
                        meta["timestamp"].as_str().unwrap_or("unknown"),
                        meta["files"].as_array().map_or(0, |a| a.len()),
                    );
                } else {
                    error!(
                        "Pool is in DIRTY state - {} cannot proceed. Run 'fsck --repair' to recover.",
                        operation
                    );
                }
            }
            return Err(eyre::eyre!(
                "Pool is DIRTY - cannot execute {}. Run fsck --repair to recover.",
                operation
            ));
        }
        Ok(())
    }

    // ---- private helpers ----

    async fn write_dirty_marker(dirty_file_path: &Path, files: Vec<String>) -> Result<()> {
        let meta = serde_json::json!({
            "files": files,
            "timestamp": Local::now().to_rfc3339(),
        });
        let content = serde_json::to_string_pretty(&meta)
            .wrap_err("Failed to serialize dirty file metadata")?;
        tokio::fs::write(dirty_file_path, content)
            .await
            .wrap_err("Failed to write dirty file")?;
        debug!("Dirty file created: {:?}", dirty_file_path);
        Ok(())
    }

    async fn check_pending_timestamps(refcnt_path: &Path, pending_files: &[PathBuf]) {
        let refcnt_mtime = match metadata(refcnt_path).await.and_then(|m| {
            m.modified()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        }) {
            Ok(t) => t,
            Err(_) => return,
        };
        for file in pending_files {
            if let Ok(file_meta) = metadata(file).await {
                if let Ok(file_mtime) = file_meta.modified() {
                    if file_mtime < refcnt_mtime {
                        warn!(
                            "Pending file {:?} is older than REFCNT — possible crash recovery scenario",
                            file.file_name().unwrap_or_default(),
                        );
                    }
                }
            }
        }
    }
}

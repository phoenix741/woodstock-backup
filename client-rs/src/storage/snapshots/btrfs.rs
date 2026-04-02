use mnt::get_mount;
use nix::sys::statfs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use eyre::{eyre, Result};
use tokio::process::Command;
use tonic::async_trait;

use crate::storage::snapshots::{SnapshotCompletion, SnapshotManager, SnapshotReference};
use woodstock;

/// Detects the mount path of the `path` parameter, provided`
fn detect_mount_path(path: &Path) -> Result<PathBuf> {
    get_mount(path)
        .map_err(|e| eyre!("Failed to get mount point for {}: {}", path.display(), e))
        .and_then(|mount| {
            mount
                .map(|mount| mount.file)
                .ok_or_else(|| eyre!("Mount point not found for {}", path.display()))
        })
}

/// BTRFS-specific snapshot reference
#[derive(Debug, Clone)]
pub struct BtrfsSnapshotReference {
    /// The path that should be used for file access (redirection target)
    redirection_path: PathBuf,
    /// The path of the snapshot root that should be deleted during cleanup
    snapshot_root_path: PathBuf,
    /// Whether sudo is required for BTRFS operations
    sudo_required: bool,
    /// Flag to track if the snapshot has been explicitly cleaned up
    /// This is used by Drop to warn about potential leaks
    cleaned_up: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl BtrfsSnapshotReference {
    /// Creates a new BTRFS snapshot reference
    ///
    /// # Arguments
    /// * `redirection_path` - The path that should be used for file access (includes the relative path from mount point)
    /// * `snapshot_root_path` - The path of the snapshot root that should be deleted during cleanup
    /// * `sudo_required` - Whether sudo is required for BTRFS operations
    pub fn new(
        redirection_path: PathBuf,
        snapshot_root_path: PathBuf,
        sudo_required: bool,
    ) -> Self {
        Self {
            redirection_path,
            snapshot_root_path,
            sudo_required,
            cleaned_up: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Gets the snapshot root path used for deletion
    pub fn snapshot_root_path(&self) -> &Path {
        &self.snapshot_root_path
    }

    /// Mark this snapshot reference as cleaned up
    ///
    /// This prevents the Drop implementation from logging a warning.
    /// This method is useful when the snapshot is cleaned up through
    /// external means (e.g., manual deletion).
    pub fn mark_cleaned_up(&self) {
        self.cleaned_up
            .store(true, std::sync::atomic::Ordering::Release);
    }

    /// Check if this snapshot reference has been marked as cleaned up
    pub fn is_cleaned_up(&self) -> bool {
        self.cleaned_up.load(std::sync::atomic::Ordering::Acquire)
    }
}

#[async_trait]
impl SnapshotReference for BtrfsSnapshotReference {
    fn path(&self) -> &Path {
        &self.redirection_path
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn finalize_self(&self, _completion: SnapshotCompletion) -> Result<()> {
        let mut cmd = if self.sudo_required {
            let mut c = Command::new("sudo");
            c.arg("btrfs");
            c
        } else {
            Command::new("btrfs")
        };

        let status = cmd
            .args(["subvolume", "delete"])
            .arg(&self.snapshot_root_path)
            .status()
            .await?;

        if !status.success() {
            return Err(eyre!(
                "Failed to delete BTRFS snapshot '{}': exit code {:?}",
                self.snapshot_root_path.display(),
                status.code()
            ));
        }

        // Mark as cleaned up to prevent Drop warning
        self.cleaned_up
            .store(true, std::sync::atomic::Ordering::Release);

        Ok(())
    }
}

/// Drop implementation to warn about potential snapshot leaks
///
/// This implementation will log a warning if a snapshot reference is dropped
/// without being explicitly cleaned up via `delete_self()`. This helps catch
/// cases where snapshots might be accidentally leaked.
impl Drop for BtrfsSnapshotReference {
    fn drop(&mut self) {
        if !self.cleaned_up.load(std::sync::atomic::Ordering::Acquire) {
            tracing::warn!(
                "BTRFS snapshot '{}' was dropped without explicit cleanup. \
                This may indicate a potential snapshot leak. Consider calling \
                delete_self() or using proper cleanup in FileSystemAccessor.",
                self.snapshot_root_path.display()
            );
        }
    }
}

pub struct BtrfsSnapshotManager {
    sudo_required: bool,
}

impl BtrfsSnapshotManager {
    pub fn new(sudo_required: bool) -> Self {
        Self { sudo_required }
    }
}

#[async_trait]
impl SnapshotManager for BtrfsSnapshotManager {
    /// Check if the source_path is a btrfs filesystem.
    async fn is_available(&self, source_path: &Path) -> Result<bool> {
        let source_path = detect_mount_path(source_path)?;

        let stat = statfs::statfs(&source_path)
            .map_err(|e| eyre!("Failed to check filesystem type with statfs: {}", e))?;

        if stat.filesystem_type() != statfs::BTRFS_SUPER_MAGIC {
            return Ok(false);
        }

        // Check btrfs subvolume support
        let mut cmd = if self.sudo_required {
            let mut c = Command::new("sudo");
            c.arg("btrfs");
            c
        } else {
            Command::new("btrfs")
        };

        let btrfs_support = cmd
            .arg("subvolume")
            .arg("snapshot")
            .arg("--help")
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false);

        Ok(btrfs_support)
    }

    async fn create_snapshot(&self, source_path: &Path) -> Result<Box<dyn SnapshotReference>> {
        let mount_path = detect_mount_path(source_path)?;
        let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
        let snapshot_name = format!(".woodstock-snapshot-{}", timestamp);
        let snapshot_root_path = mount_path.join(&snapshot_name);

        // Calculate the relative path from mount point to the source path
        let relative_path = source_path
            .strip_prefix(&mount_path)
            .map_err(|e| eyre!("Failed to compute relative path from mount point: {}", e))?;

        // The redirection path is the snapshot root + the relative path
        let redirection_path = snapshot_root_path.join(relative_path);

        let mut cmd = if self.sudo_required {
            let mut c = Command::new("sudo");
            c.arg("btrfs");
            c
        } else {
            Command::new("btrfs")
        };

        let status = cmd
            .args(["subvolume", "snapshot", "-r"])
            .arg(&mount_path)
            .arg(&snapshot_root_path)
            .status()
            .await?;

        if !status.success() {
            return Err(eyre!(
                "Failed to create BTRFS snapshot: exit code {:?}",
                status.code()
            ));
        }

        Ok(Box::new(BtrfsSnapshotReference::new(
            redirection_path,
            snapshot_root_path,
            self.sudo_required,
        )))
    }

    fn manager_name(&self) -> &'static str {
        "BTRFS"
    }

    fn snapshot_method(&self) -> woodstock::SnapshotMethod {
        woodstock::SnapshotMethod::Btrfs
    }

    fn priority(&self) -> u8 {
        120 // Higher priority for BTRFS
    }
}

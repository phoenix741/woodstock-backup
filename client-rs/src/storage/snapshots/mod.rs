//! Filesystem Snapshot Management
//!
//! This module provides a robust and extensible system for managing filesystem snapshots
//! during backup operations. The system is designed to support multiple snapshot backends
//! (BTRFS, ZFS, VSS, etc.) and ensures reliable cleanup without relying on snapshot
//! manager re-detection.
//!
//! ## Architecture
//!
//! The snapshot system is built around two main traits:
//!
//! - [`SnapshotManager`]: Responsible for creating snapshots and checking availability
//! - [`SnapshotReference`]: Represents a snapshot with self-cleanup capabilities
//!
//! ## Key Design Principles
//!
//! ### Self-Contained Cleanup
//! Each snapshot reference contains all the information needed to delete itself,
//! eliminating the need to re-detect the snapshot manager during cleanup. This
//! makes the system more robust and reliable.
//!
//! ### Simplified API
//! The removal of the `delete_snapshot` method from `SnapshotManager` simplifies
//! the API and forces the use of the more robust `delete_self()` method.
//!
//! ### Drop Safety
//! Snapshot references implement [`Drop`] to warn about potential leaks when
//! snapshots are dropped without explicit cleanup.
//!
//! ### Extensibility
//! The trait-based design allows easy addition of new snapshot backends without
//! modifying existing code.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use woodstock_client_rs::storage::snapshots::{select_snapshot_manager, SnapshotReference};
//! use std::path::Path;
//!
//! # #[tokio::main]
//! # async fn main() -> eyre::Result<()> {
//! let source_path = Path::new("/path/to/backup");
//!
//! // Find the best snapshot manager for this path
//! if let Some(manager) = select_snapshot_manager(source_path).await {
//!     // Create a snapshot
//!     let snapshot = manager.create_snapshot(source_path).await?;
//!     
//!     // Use the snapshot for backup operations
//!     println!("Snapshot available at: {}", snapshot.path().display());
//!     
//!     // Clean up the snapshot (self-contained deletion)
//!     snapshot.delete_self().await?;
//! }
//! # Ok(())
//! # }
//! ```

#[cfg(unix)]
pub mod btrfs;
#[cfg(windows)]
pub mod vss;

use std::path::Path;

use eyre::Result;
use tonic::async_trait;

/// Describes how a snapshot-backed backup session ended.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotCompletion {
    /// The backup reached its nominal end and backend-specific completion hooks should run.
    Success,
    /// The backup ended prematurely and the backend should abort its work before cleanup.
    Abort,
}

/// Manages filesystem snapshots for backup operations
#[async_trait]
pub trait SnapshotManager: Send + Sync {
    /// Create a new snapshot of the specified path
    async fn create_snapshot(&self, source_path: &Path) -> Result<Box<dyn SnapshotReference>>;

    /// Check if this snapshot manager is available for the given path
    async fn is_available(&self, source_path: &Path) -> Result<bool>;

    /// Clean up all active snapshots managed by this instance
    async fn cleanup_all(&self) -> Result<()> {
        // Default implementation does nothing
        Ok(())
    }

    /// Get the name/identifier of this snapshot manager
    fn manager_name(&self) -> &'static str;

    /// Get the priority of this manager (higher = preferred)
    fn priority(&self) -> u8 {
        100 // Default priority
    }
}

/// Represents a reference to a snapshot
#[async_trait]
pub trait SnapshotReference: Send + Sync {
    /// Get the path to the snapshot
    fn path(&self) -> &Path;

    /// Get a string representation of the snapshot reference
    fn as_string(&self) -> String {
        self.path().to_string_lossy().to_string()
    }

    /// Enable downcasting to concrete types for manager-specific operations
    fn as_any(&self) -> &dyn std::any::Any;

    /// Finalize this snapshot according to how the backup ended.
    async fn finalize_self(&self, completion: SnapshotCompletion) -> Result<()>;

    /// Delete this snapshot using the manager that created it.
    ///
    /// This method allows the snapshot reference to delete itself without requiring
    /// re-detection of the snapshot manager. Each implementation stores the necessary
    /// information to perform the deletion (manager type, sudo requirements, etc.).
    ///
    /// # Returns
    /// * `Ok(())` if the snapshot was deleted successfully
    /// * `Err(...)` if the deletion failed
    async fn delete_self(&self) -> Result<()> {
        self.finalize_self(SnapshotCompletion::Abort).await
    }
}

/// Selects the best available snapshot manager for the given path
///
/// This function evaluates all available snapshot managers in order of priority
/// and returns the first one that is available for the specified path.
///
/// # Arguments
/// * `source_path` - The path to check for snapshot support
///
/// # Returns
/// * `Some(Box<dyn SnapshotManager>)` if a suitable manager is found
/// * `None` if no snapshot manager is available for this path
pub async fn select_snapshot_manager<P: AsRef<Path>>(
    source_path: P,
) -> Option<Box<dyn SnapshotManager>> {
    let source_path = source_path.as_ref();

    // Collect all available managers
    let mut managers: Vec<Box<dyn SnapshotManager>> = Vec::new();

    #[cfg(unix)]
    {
        // Add BTRFS manager
        managers.push(Box::new(btrfs::BtrfsSnapshotManager::new(false)));

        // TODO: Add ZFS support for Unix systems
        // managers.push(Box::new(zfs::ZfsSnapshotManager::new()));
    }

    #[cfg(windows)]
    {
        managers.push(Box::new(vss::VssSnapshotManager::new()));
    }

    // Sort managers by priority (highest first)
    managers.sort_by(|a, b| b.priority().cmp(&a.priority()));

    // Find the first available manager
    for manager in managers {
        if manager.is_available(source_path).await.unwrap_or(false) {
            return Some(manager);
        }
    }

    None
}

/// Get all available snapshot managers for the system
///
/// This function returns a list of all snapshot managers that could potentially
/// be used, regardless of whether they're available for a specific path.
///
/// # Returns
/// A vector of all compiled snapshot managers for this platform
pub fn get_available_managers() -> Vec<Box<dyn SnapshotManager>> {
    let mut managers: Vec<Box<dyn SnapshotManager>> = Vec::new();

    #[cfg(unix)]
    {
        managers.push(Box::new(btrfs::BtrfsSnapshotManager::new(false)));
        // TODO: Add other Unix managers (ZFS, etc.)
    }

    #[cfg(windows)]
    {
        managers.push(Box::new(vss::VssSnapshotManager::new()));
    }

    // Sort by priority
    managers.sort_by(|a, b| b.priority().cmp(&a.priority()));
    managers
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_select_snapshot_manager_with_no_available_managers() {
        // Create a temporary directory that won't be on BTRFS
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path();

        // Should return None since no snapshot managers are available for this path
        let manager = select_snapshot_manager(test_path).await;
        assert!(manager.is_none());
    }

    #[test]
    fn test_get_available_managers() {
        let managers = get_available_managers();

        // Should have at least the BTRFS manager on Unix systems
        #[cfg(unix)]
        {
            assert!(!managers.is_empty());
            assert_eq!(managers[0].manager_name(), "BTRFS");
            assert_eq!(managers[0].priority(), 120);
        }

        // On Windows, VSS should be available as a compiled manager.
        #[cfg(windows)]
        {
            assert!(!managers.is_empty());
            assert_eq!(managers[0].manager_name(), "VSS");
        }
    }

    #[tokio::test]
    async fn test_snapshot_manager_trait_methods() {
        #[cfg(unix)]
        {
            let manager = btrfs::BtrfsSnapshotManager::new(false);

            // Test manager properties
            assert_eq!(manager.manager_name(), "BTRFS");
            assert_eq!(manager.priority(), 120);

            // Test cleanup_all (should not fail)
            let result = manager.cleanup_all().await;
            assert!(result.is_ok());
        }

        #[cfg(windows)]
        {
            let manager = vss::VssSnapshotManager::new();

            assert_eq!(manager.manager_name(), "VSS");
            assert_eq!(manager.priority(), 110);

            let result = manager.cleanup_all().await;
            assert!(result.is_ok());
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_btrfs_snapshot_reference() {
        let redirection_path = PathBuf::from("/test/snapshot/path");
        let snapshot_root_path = PathBuf::from("/test/snapshot");
        let reference = btrfs::BtrfsSnapshotReference::new(
            redirection_path.clone(),
            snapshot_root_path.clone(),
            false, // sudo not required for tests
        );

        assert_eq!(reference.path(), redirection_path.as_path());
        assert_eq!(reference.as_string(), "/test/snapshot/path");
        assert_eq!(reference.snapshot_root_path(), snapshot_root_path.as_path());
    }

    #[test]
    #[cfg(unix)]
    fn test_btrfs_snapshot_path_calculation() {
        // Simulate a scenario where we have /home/phoenix/Documents
        // Mount point is /home
        // Snapshot should be created at /home/.woodstock-snapshot-xxx/
        // But redirection should point to /home/.woodstock-snapshot-xxx/phoenix/Documents

        let redirection_path =
            PathBuf::from("/home/.woodstock-snapshot-20250623-123456/phoenix/Documents");
        let snapshot_root_path = PathBuf::from("/home/.woodstock-snapshot-20250623-123456");

        let reference = btrfs::BtrfsSnapshotReference::new(
            redirection_path.clone(),
            snapshot_root_path.clone(),
            false, // sudo not required for tests
        );

        // The path() method should return the redirection path for file operations
        assert_eq!(reference.path(), redirection_path.as_path());

        // The snapshot_root_path() method should return the root path for deletion
        assert_eq!(reference.snapshot_root_path(), snapshot_root_path.as_path());

        // Verify the redirection path ends with the original relative path
        assert!(reference
            .path()
            .to_string_lossy()
            .ends_with("phoenix/Documents"));

        // Verify the snapshot root path is the parent of the redirection path
        assert_eq!(
            reference.path().parent().unwrap().parent().unwrap(),
            reference.snapshot_root_path()
        );
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_btrfs_snapshot_cleanup_tracking() {
        use std::path::PathBuf;

        let redirection_path = PathBuf::from("/tmp/test-snapshot/data");
        let snapshot_root_path = PathBuf::from("/tmp/test-snapshot");

        let reference =
            btrfs::BtrfsSnapshotReference::new(redirection_path, snapshot_root_path, false);

        // Initially, snapshot should not be cleaned up
        assert!(!reference.is_cleaned_up());

        // Mark as cleaned up manually
        reference.mark_cleaned_up();
        assert!(reference.is_cleaned_up());

        // When dropped now, it should NOT warn (because it's marked as cleaned up)
        drop(reference);

        // Test the self-deletion cleanup tracking
        let redirection_path2 = PathBuf::from("/tmp/test-snapshot2/data");
        let snapshot_root_path2 = PathBuf::from("/tmp/test-snapshot2");

        let reference2 =
            btrfs::BtrfsSnapshotReference::new(redirection_path2, snapshot_root_path2, false);

        assert!(!reference2.is_cleaned_up());

        // Simulate successful deletion via delete_self
        // Note: This would fail in real execution because the path doesn't exist,
        // but it demonstrates the cleanup tracking mechanism
        let result: eyre::Result<()> = reference2.delete_self().await;

        // Even if deletion fails, let's test the tracking mechanism independently
        if result.is_ok() {
            assert!(reference2.is_cleaned_up());
        }
    }
}

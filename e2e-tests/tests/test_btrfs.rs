//! End-to-end test for BtrfsSnapshotManager and FileSystemAccessor integration
// This test requires a Btrfs filesystem and may need root privileges.
// It will be skipped if Btrfs is not available or the environment is not suitable.

use eyre::Result;
use woodstock_client_rs::storage::snapshots::btrfs::BtrfsSnapshotManager;
use woodstock_client_rs::storage::snapshots::SnapshotManager;

#[tokio::test]
#[cfg(unix)]
async fn test_btrfs_snapshot_manager_e2e() -> Result<()> {
    // Try to find a Btrfs mount point (commonly / or /home)
    let share_path = "/home/phoenix";
    let manager = BtrfsSnapshotManager::new(true);
    let is_available = manager
        .is_available(std::path::Path::new(share_path))
        .await?;

    if !is_available {
        eprintln!("No suitable Btrfs mount point found, skipping test.");
        return Ok(());
    }

    // Create a snapshot
    let snapshot_ref = manager
        .create_snapshot(std::path::Path::new(share_path))
        .await?;
    let snapshot_path = snapshot_ref.path();

    if !snapshot_path.exists() {
        eprintln!(
            "Snapshot path does not exist: {:?}. Skipping test.",
            snapshot_path
        );
        return Err(eyre::eyre!(
            "Snapshot path does not exist: {:?}",
            snapshot_path
        ));
    }

    // Delete the snapshot using the robust self-deletion method
    snapshot_ref.delete_self().await?;
    if snapshot_path.exists() {
        eprintln!(
            "Snapshot path still exists after deletion: {:?}. Skipping test.",
            snapshot_path
        );
        return Err(eyre::eyre!(
            "Snapshot path still exists after deletion: {:?}",
            snapshot_path
        ));
    }

    Ok(())
}

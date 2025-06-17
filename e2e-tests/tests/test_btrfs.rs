//! End-to-end test for BtrfsSnapshotManager
// This test requires a Btrfs filesystem and may need root privileges.
// It will be skipped if Btrfs is not available or the environment is not suitable.

use eyre::Result;
use woodstock_client_rs::snapshots::btrfs::BtrfsSnapshotManager;
use woodstock_client_rs::snapshots::SnapshotManager;

#[tokio::test]
#[cfg(unix)]
async fn test_btrfs_snapshot_manager_e2e() -> Result<()> {
    // Try to find a Btrfs mount point (commonly / or /home)
    let share_path = "/home/phoenix"; // Add more if needed
    let manager = BtrfsSnapshotManager::new(true);
    let is_available = manager.is_available(share_path).await?;

    if !is_available {
        eprintln!("No suitable Btrfs mount point found, skipping test.");
        return Ok(());
    }

    // Create a snapshot
    let snapshot = manager.create_snapshot(&share_path).await?;
    if !snapshot.exists() {
        eprintln!(
            "Snapshot path does not exist: {:?}. Skipping test.",
            snapshot
        );
        return Err(eyre::eyre!("Snapshot path does not exist: {:?}", snapshot));
    }

    // Delete the snapshot using the robust self-deletion method
    snapshot_ref.delete_self().await?;
    if snapshot.exists() {
        eprintln!(
            "Snapshot path still exists after deletion: {:?}. Skipping test.",
            snapshot
        );
        return Err(eyre::eyre!(
            "Snapshot path still exists after deletion: {:?}",
            snapshot
        ));
    }

    Ok(())
}

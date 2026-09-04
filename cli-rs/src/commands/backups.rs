//! Commands for reconciling `hosts/<host>/<uuid>/` directories against `backup.yml`.
//!
//! A backup directory can end up on disk with no corresponding entry in its host's
//! `backup.yml` (crash between directory creation and the first tracking write, or a
//! corrupt `backup.yml` silently rewritten as an empty list — see the fixes in
//! `woodstock-rs/src/config/backups.rs` and `save.rs`). Nothing ever resumes or removes
//! such a directory automatically, since every resume/removal path keys off `backup.yml`.
//! This module finds and, on request, removes them.

use std::path::{Path, PathBuf};

use eyre::Result;
use indicatif::HumanBytes;
use tokio::fs::{read_dir, remove_dir_all};
use tracing::{info, warn};
use uuid::Uuid;

use crate::commands::CliServiceState;

/// A `hosts/<host>/<uuid>/` directory with no matching entry in that host's `backup.yml`.
struct OrphanBackupDir {
    hostname: String,
    backup_id: Uuid,
    path: PathBuf,
    size: u64,
}

/// Scans every host under `hosts_path` for backup directories absent from `backup.yml`,
/// and either reports them (`dry_run`) or deletes them.
///
/// # Errors
///
/// Returns an error if the hosts directory cannot be listed, or if removing an orphan
/// directory fails.
pub async fn clean_orphan_backups(state: CliServiceState, dry_run: bool) -> Result<()> {
    let orphans = find_orphan_backups(&state).await?;

    if orphans.is_empty() {
        info!("No orphan backup directory found — nothing to do.");
        return Ok(());
    }

    let total_size: u64 = orphans.iter().map(|o| o.size).sum();
    for orphan in &orphans {
        info!(
            "{}/{} — {} — {:?}",
            orphan.hostname,
            orphan.backup_id,
            HumanBytes(orphan.size),
            orphan.path
        );
    }
    info!(
        "{} orphan backup director{} found, {} total{}",
        orphans.len(),
        if orphans.len() == 1 { "y" } else { "ies" },
        HumanBytes(total_size),
        if dry_run {
            " (dry run — nothing removed, re-run with --no-dry-run to delete)"
        } else {
            ""
        }
    );

    if dry_run {
        return Ok(());
    }

    for orphan in &orphans {
        match remove_dir_all(&orphan.path).await {
            Ok(()) => info!("Removed {:?}", orphan.path),
            Err(e) => warn!("Failed to remove {:?}: {}", orphan.path, e),
        }
    }

    Ok(())
}

/// Finds every backup directory on disk that has no corresponding entry in its host's
/// `backup.yml`.
async fn find_orphan_backups(state: &CliServiceState) -> Result<Vec<OrphanBackupDir>> {
    let hosts_path = &state.config.path.hosts_path;
    let mut orphans = Vec::new();

    let mut host_entries = read_dir(hosts_path).await?;
    while let Some(host_entry) = host_entries.next_entry().await? {
        if !host_entry.file_type().await?.is_dir() {
            continue;
        }
        let Some(hostname) = host_entry.file_name().to_str().map(str::to_string) else {
            continue;
        };

        let tracked_ids: std::collections::HashSet<Uuid> = state
            .backups
            .get_backups(&hostname)
            .await
            .into_iter()
            .map(|b| b.id)
            .collect();

        let mut backup_entries = read_dir(host_entry.path()).await?;
        while let Some(backup_entry) = backup_entries.next_entry().await? {
            if !backup_entry.file_type().await?.is_dir() {
                continue;
            }
            let Some(name) = backup_entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            let Ok(backup_id) = Uuid::parse_str(&name) else {
                // Not a UUID-named directory (e.g. stray files) — not a backup dir.
                continue;
            };
            if tracked_ids.contains(&backup_id) {
                continue;
            }

            let path = backup_entry.path();
            let size = host_backup_dir_size(&path).await.unwrap_or(0);
            orphans.push(OrphanBackupDir {
                hostname: hostname.clone(),
                backup_id,
                path,
                size,
            });
        }
    }

    Ok(orphans)
}

/// Sums the size of the files directly under `path`. A backup directory
/// (`hosts/<host>/<uuid>/`) is flat — manifest, REFCNT, logs, `shares.yml` — no
/// subdirectories, so no need to walk recursively.
async fn host_backup_dir_size(path: &Path) -> Result<u64> {
    let mut total = 0u64;
    let mut entries = read_dir(path).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_file() {
            total += entry.metadata().await?.len();
        }
    }
    Ok(total)
}

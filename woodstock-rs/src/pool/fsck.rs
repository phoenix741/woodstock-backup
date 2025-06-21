use std::{path::PathBuf, time::SystemTime};

use async_walkdir::{Filtering, WalkDir};
use eyre::Result;
use futures::{pin_mut, StreamExt};
use log::{error, info};
use tokio::sync::mpsc;

use crate::{
    config::{Backups, Configuration},
    pool::PoolChunkWrapper,
    PoolRefCount,
};

use super::Refcnt;

/// # Pool Filesystem Check (fsck) Module
///
/// This module provides functions for verifying and repairing the integrity of the Woodstock backup pool.
/// It includes checks for backup integrity, host integrity, pool integrity, and unused chunks.
///
/// ## Main Functions
///
/// - [`check_backup_integrity`]: Verifies the integrity of a specific backup.
/// - [`check_host_integrity`]: Verifies the integrity of all backups for a specific host.
/// - [`check_pool_integrity`]: Verifies the integrity of the entire pool.
/// - [`check_unused`]: Identifies and manages unused chunks in the pool.
///
/// ## Error Handling & Panics
///
/// - All functions return `Result` and propagate errors using the `eyre` crate.
/// - Panics are not expected under normal operation.
///
/// ## See Also
///
/// - [`Refcnt`], [`PoolChunkWrapper`]: For related pool operations
pub struct FsckCount {
    pub error_count: usize,
    pub total_count: usize,
}

#[derive(Debug, Clone)]
pub struct FsckUnusedCount {
    pub in_unused: usize,
    pub in_refcnt: usize,
    pub in_nothing: usize,
    pub missing: usize,
}

/// Checks the integrity of the reference counts.
///
/// # Arguments
/// * `original_refcnt` - The original reference count data.
/// * `new_refcnt` - The new reference count data to compare against.
///
/// # Returns
/// The number of errors found during the integrity check.
fn check_integrity(original_refcnt: &Refcnt, new_refcnt: &Refcnt, path: &PathBuf) -> usize {
    let mut error_count = 0;

    for refcnt in new_refcnt.list_refcnt() {
        if let Some(original_refcnt) = original_refcnt.get_refcnt(&refcnt.sha256) {
            if original_refcnt.ref_count != refcnt.ref_count {
                error_count += 1;
                error!(
                    "Refcnt mismatch {}: {} instead of {} in {:?}",
                    hex::encode(&refcnt.sha256),
                    original_refcnt.ref_count,
                    refcnt.ref_count,
                    path
                );
            }
        } else if refcnt.ref_count > 0 {
            error_count += 1;
            error!(
                "Refcnt not in original {}: must be {} in {:?}",
                hex::encode(&refcnt.sha256),
                refcnt.ref_count,
                path
            );
        }
    }

    error_count
}

/// Verifies the integrity of a specific backup.
///
/// # Arguments
///
/// * `hostname` - The hostname associated with the backup.
/// * `backup_number` - The backup number to check.
/// * `dry_run` - If true, do not modify any data.
/// * `config` - Reference to the Woodstock [`Configuration`] struct.
///
/// # Returns
///
/// * `Ok(FsckCount)` - The result of the integrity check (error count and total count).
/// * `Err(eyre::Report)` if an error occurs during the integrity check.
///
/// # Errors
///
/// Returns an error if the integrity check fails.
pub async fn check_backup_integrity(
    hostname: &str,
    backup_number: usize,
    dry_run: bool,
    config: &Configuration,
) -> Result<FsckCount> {
    let backups = Backups::new(config);
    let destination_backup = backups.get_backup_destination_directory(hostname, backup_number);

    let new_refcnt =
        Refcnt::new_backup_refcnt_from_manifest(hostname, backup_number, config).await?;
    let mut original_refcnt = Refcnt::new(&destination_backup);
    original_refcnt.load_refcnt(false).await;

    let error_count = check_integrity(&original_refcnt, &new_refcnt, &destination_backup);

    let mut new_refcnt = new_refcnt;

    if !dry_run && error_count > 0 {
        info!("Fix refcnt for {hostname}/{backup_number}");
        new_refcnt.repair(&config.path.pool_path, config.chunk_algorithm).await?;
        new_refcnt.save_refcnt(&SystemTime::now(), false).await?;
    }

    Ok(FsckCount {
        error_count,
        total_count: new_refcnt.size(),
    })
}

/// Verifies the integrity of all backups for a specific host.
///
/// # Arguments
///
/// * `hostname` - The hostname to check.
/// * `dry_run` - If true, do not modify any data.
/// * `config` - Reference to the Woodstock [`Configuration`] struct.
///
/// # Returns
///
/// * `Ok(FsckCount)` - The result of the integrity check (error count and total count).
/// * `Err(eyre::Report)` if an error occurs.
///
/// # Errors
///
/// Returns an error if the host backups cannot be loaded or the integrity check fails.
pub async fn check_host_integrity(
    hostname: &str,
    dry_run: bool,
    config: &Configuration,
) -> Result<FsckCount> {
    let backups = Backups::new(config);
    let destination_directory = backups.get_host_path(hostname);

    let new_refcnt = Refcnt::new_host_refcnt_from_backups(hostname, config).await?;

    let mut original_refcnt = Refcnt::new(&destination_directory);
    original_refcnt.load_refcnt(false).await;

    let error_count = check_integrity(&original_refcnt, &new_refcnt, &destination_directory);

    let mut new_refcnt = new_refcnt;

    if !dry_run && error_count > 0 {
        info!("Fix refcnt for {hostname}");
        new_refcnt.repair(&config.path.pool_path, config.chunk_algorithm).await?;
        new_refcnt.save_refcnt(&SystemTime::now(), false).await?;
    }

    Ok(FsckCount {
        error_count,
        total_count: new_refcnt.size(),
    })
}

/// Verifies the integrity of the entire pool.
///
/// # Arguments
///
/// * `dry_run` - If true, do not modify any data.
/// * `config` - Reference to the Woodstock [`Configuration`] struct.
///
/// # Returns
///
/// * `Ok(FsckCount)` - The result of the integrity check (error count and total count).
/// * `Err(eyre::Report)` if an error occurs.
///
/// # Errors
///
/// Returns an error if the pool cannot be loaded or the integrity check fails.
pub async fn check_pool_integrity(dry_run: bool, config: &Configuration) -> Result<FsckCount> {
    let mut pool_refcnt = Refcnt::new(&config.path.pool_path);
    pool_refcnt.load_refcnt(false).await;

    let new_refcnt = Refcnt::new_pool_refcnt_from_host(config).await?;

    let error_count = check_integrity(&pool_refcnt, &new_refcnt, &config.path.pool_path);

    let mut new_refcnt = new_refcnt;

    if !dry_run && error_count > 0 {
        info!("Fix refcnt for pool");
        new_refcnt.repair(&config.path.pool_path, config.chunk_algorithm).await?;
        new_refcnt.save_refcnt(&SystemTime::now(), true).await?;
    }

    Ok(FsckCount {
        error_count,
        total_count: new_refcnt.size(),
    })
}

/// Identifies and manages unused chunks in the pool.
///
/// # Arguments
///
/// * `dry_run` - If true, do not modify any data.
/// * `progress_tx` - Channel for progress updates.
/// * `config` - Reference to the Woodstock [`Configuration`] struct.
///
/// # Returns
///
/// * `Ok(FsckUnusedCount)` - The result of the unused chunk check.
/// * `Err(eyre::Report)` if an error occurs.
///
/// # Errors
///
/// Returns an error if the pool cannot be loaded or the unused chunk check fails.
///
/// # Panics
///
/// This function will panic if the `progress_tx` channel is closed unexpectedly. Ensure that the channel is properly managed and remains open during the operation.
pub async fn check_unused(
    dry_run: bool,
    progress_tx: mpsc::Sender<FsckUnusedCount>,
    config: &Configuration,
) -> Result<FsckUnusedCount> {
    let mut pool_refcnt = Refcnt::new(&config.path.pool_path);
    pool_refcnt.load_refcnt(false).await;

    let mut count = FsckUnusedCount {
        in_unused: 0,
        in_refcnt: 0,
        in_nothing: 0,
        missing: 0,
    };

    // FIXME: Remove walkdir and use unfold like get_files_recursive
    let entries = WalkDir::new(&config.path.pool_path)
        .filter(|f| async move {
            let path = f.path();

            if path.extension().is_some_and(|ext| ext == "zz") {
                Filtering::Continue
            } else {
                Filtering::Ignore
            }
        })
        .filter_map(|path| async move {
            if let Ok(path) = path {
                // Remove -sha256.zz from filename
                let filename = path.file_name().into_string().ok().unwrap_or_default();
                return hex::decode(&filename[..64]).ok();
            }
            None
        });
    pin_mut!(entries);

    while let Some(hash) = entries.next().await {
        let wrapper = PoolChunkWrapper::new(&config.path.pool_path, Some(&hash));
        let hash_str = wrapper.get_hash_str().as_ref().unwrap();

        let refcnt = pool_refcnt.get_refcnt(&hash);
        if refcnt.is_some_and(|r| r.ref_count == 0) {
            count.in_unused += 1;
        } else if refcnt.is_some_and(|r| r.ref_count > 0) {
            count.in_refcnt += 1;
            if let Err(e) = progress_tx.send(count.clone()).await {
                error!("Failed to send progress update: {}", e);
            }
        } else {
            let information = wrapper.chunk_information().await?;

            count.in_nothing += 1;
            pool_refcnt.apply(
                &PoolRefCount {
                    sha256: hash.clone(),
                    size: information.size,
                    compressed_size: information.compressed_size,
                    ref_count: 0,
                },
                &crate::pool::RefcntApplySens::Increase,
            );
            error!("{} is not in unused nor in refcnt", hash_str);
        }
    }

    for refcnt in pool_refcnt.list_refcnt() {
        let wrapper = PoolChunkWrapper::new(&config.path.pool_path, Some(&refcnt.sha256));
        if !wrapper.exists() {
            count.missing += 1;
            error!(
                "{} is missing (refcnt {})",
                hex::encode(&refcnt.sha256),
                refcnt.ref_count
            );
        }
    }

    if !dry_run {
        pool_refcnt.save_refcnt(&SystemTime::now(), true).await?;
    }

    Ok(count)
}

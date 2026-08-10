use std::{path::PathBuf, sync::Arc};

use async_walkdir::{Filtering, WalkDir};
use chrono::Local;
use eyre::Result;
use futures::{pin_mut, StreamExt};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::{
    config::{Backups, Configuration, Hosts},
    pool::PoolChunkWrapper,
    proto::{CompressedWriter, ProtobufReader, ProtobufWriter},
    PoolRefCount, PoolUnused,
};
use uuid::Uuid;

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
/// * `backup_id` - The UUID v7 identifier of the backup to check.
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
    backup_id: Uuid,
    dry_run: bool,
    config: Arc<Configuration>,
    backups: Arc<Backups>,
) -> Result<FsckCount> {
    let destination_backup = backups.get_backup_destination_directory(hostname, backup_id);

    let new_refcnt =
        Refcnt::new_backup_refcnt_from_manifest(hostname, backup_id, config.clone()).await?;
    let mut original_refcnt = Refcnt::new(&destination_backup);
    original_refcnt.load_refcnt(false).await;

    let error_count = check_integrity(&original_refcnt, &new_refcnt, &destination_backup);

    let mut new_refcnt = new_refcnt;

    if !dry_run && error_count > 0 {
        info!("Fix refcnt for {hostname}/{backup_id}");
        new_refcnt
            .repair(&config.path.pool_path, config.chunk_algorithm)
            .await?;
        new_refcnt
            .save_refcnt(&Local::now(), false, config.compression_format)
            .await?;
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
    config: Arc<Configuration>,
    backups: Arc<Backups>,
) -> Result<FsckCount> {
    let destination_directory = backups.get_host_path(hostname);

    let new_refcnt = Refcnt::new_host_refcnt_from_backups(hostname, config.clone()).await?;

    let mut original_refcnt = Refcnt::new(&destination_directory);
    original_refcnt.load_refcnt(false).await;

    let error_count = check_integrity(&original_refcnt, &new_refcnt, &destination_directory);

    let mut new_refcnt = new_refcnt;

    if !dry_run && error_count > 0 {
        info!("Fix refcnt for {hostname}");
        new_refcnt
            .repair(&config.path.pool_path, config.chunk_algorithm)
            .await?;
        new_refcnt
            .save_refcnt(&Local::now(), false, config.compression_format)
            .await?;
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
pub async fn check_pool_integrity(
    dry_run: bool,
    config: Arc<Configuration>,
    hosts_config: Arc<Hosts>,
    backups_config: Arc<Backups>,
) -> Result<FsckCount> {
    use super::PoolManager;

    // Recalculate REFCNT from backups (source of truth)
    let mut pool_refcnt = Refcnt::new(&config.path.pool_path);
    pool_refcnt.load_refcnt(false).await;

    let new_refcnt =
        Refcnt::new_pool_refcnt_from_host(config.clone(), hosts_config, backups_config).await?;

    let mut error_count = check_integrity(&pool_refcnt, &new_refcnt, &config.path.pool_path);

    let mut new_refcnt = new_refcnt;

    let dirty = PoolManager::new(config.clone()).is_dirty().await;
    if dirty.is_err() {
        error!(
            "Failed to check dirty state of pool: {}",
            // SAFETY: dirty.is_err() is true — unwrapping Err is always Some
            dirty.err().unwrap()
        );
        error_count += 1;
    }

    if !dry_run && error_count > 0 {
        info!("Fix refcnt for pool");
        PoolManager::new(config.clone()).cleanup_dirty().await?;
        new_refcnt
            .repair(&config.path.pool_path, config.chunk_algorithm)
            .await?;
        new_refcnt
            .save_refcnt(&Local::now(), true, config.compression_format)
            .await?;
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
    config: Arc<Configuration>,
    cancel_token: &CancellationToken,
) -> Result<FsckUnusedCount> {
    let mut pool_refcnt = Refcnt::new(&config.path.pool_path);
    pool_refcnt.load_refcnt(false).await;

    let missing_path = config.path.pool_path.join("missing");
    let mut previously_missing: std::collections::HashSet<Vec<u8>> =
        std::collections::HashSet::new();
    if let Ok(mut reader) = ProtobufReader::<PoolUnused>::new(&missing_path, true).await {
        let messages = reader.into_stream();
        pin_mut!(messages);
        while let Some(missing) = messages.next().await {
            if let Ok(missing) = missing {
                previously_missing.insert(missing.sha256);
            }
        }
    }

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

    let mut cancelled = false;
    while let Some(hash) = entries.next().await {
        if cancel_token.is_cancelled() {
            cancelled = true;
            break;
        }

        let wrapper = PoolChunkWrapper::new(&config.path.pool_path, Some(&hash));
        // SAFETY: wrapper was created with Some(&hash) — get_hash_str() always returns Some
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

    let mut still_missing = Vec::new();
    for refcnt in pool_refcnt.list_refcnt() {
        let wrapper = PoolChunkWrapper::new(&config.path.pool_path, Some(&refcnt.sha256));
        if !wrapper.exists() {
            count.missing += 1;
            if previously_missing.contains(&refcnt.sha256) {
                warn!(
                    "{} is missing (refcnt {}) — already known missing since last check",
                    hex::encode(&refcnt.sha256),
                    refcnt.ref_count
                );
            } else {
                error!(
                    "{} is missing (refcnt {})",
                    hex::encode(&refcnt.sha256),
                    refcnt.ref_count
                );
            }
            still_missing.push(PoolUnused {
                sha256: refcnt.sha256.clone(),
                size: refcnt.size,
                compressed_size: refcnt.compressed_size,
            });
        }
    }

    // Refresh the durable `missing` snapshot every run (informational, not a
    // mutation of pool state) so the next check can tell a recurring miss
    // from a new one, regardless of dry_run.
    if let Ok(mut missing_writer) =
        ProtobufWriter::<CompressedWriter, PoolUnused>::new_compressed(
            &missing_path,
            true,
            config.compression_format,
        )
        .await
    {
        for entry in &still_missing {
            if let Err(e) = missing_writer.write(entry).await {
                error!("Failed to write missing-chunk entry to {missing_path:?}: {e}");
            }
        }
        if let Err(e) = missing_writer.flush().await {
            error!("Failed to save missing-chunks file {missing_path:?}: {e}");
        }
    } else {
        error!("Failed to open missing-chunks file for writing: {missing_path:?}");
    }

    // A cancelled walk skips the save entirely — the in-memory `pool_refcnt`
    // built from a partial walk is not a trustworthy source of truth to
    // persist, and leaving the on-disk state untouched means a cancelled
    // unused-check is a pure no-op (dry-run or not), safe to just re-run.
    if !dry_run && !cancelled {
        pool_refcnt
            .save_refcnt(&Local::now(), true, config.compression_format)
            .await?;
    }

    Ok(count)
}

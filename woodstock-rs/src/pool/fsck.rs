use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
};

use async_walkdir::{Filtering, WalkDir};
use chrono::Local;
use eyre::Result;
use futures::{pin_mut, StreamExt};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::{
    config::{Backups, Configuration, Hosts, FSCK_PROGRESS_BATCH_SIZE},
    pool::PoolChunkWrapper,
    proto::{CompressedWriter, ProtobufReader, ProtobufWriter},
    utils::compression::CompressionFormat,
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
}

#[derive(Debug, Clone, Default)]
pub struct FsckMissingCount {
    pub checked: usize,
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

/// Loads the durable "missing chunks" snapshot at `<pool_path>/missing` into memory, keyed by
/// chunk hash. This snapshot is written by [`check_unused`] and lists chunks referenced by the
/// pool's refcnt but absent from disk; it lets a caller (another fsck run, or a backup process
/// opportunistically healing entries as it re-confirms chunk presence) distinguish a recurring
/// miss from a new one, or know what's still missing after a partial fix.
///
/// # Errors
///
/// This function does not return errors: a missing or unreadable snapshot file simply yields an
/// empty map (nothing known to be missing yet).
pub async fn load_missing_chunks(pool_path: &Path) -> HashMap<Vec<u8>, PoolUnused> {
    let missing_path = pool_path.join("missing");
    let mut missing = HashMap::new();

    if let Ok(mut reader) = ProtobufReader::<PoolUnused>::new(&missing_path, true).await {
        let messages = reader.into_stream();
        pin_mut!(messages);
        while let Some(Ok(entry)) = messages.next().await {
            missing.insert(entry.sha256.clone(), entry);
        }
    }

    missing
}

/// Persists `missing` as the durable "missing chunks" snapshot at `<pool_path>/missing`,
/// replacing whatever was there before.
///
/// # Errors
///
/// Returns an error if the snapshot file cannot be created or written to.
pub async fn save_missing_chunks<'a>(
    pool_path: &Path,
    missing: impl Iterator<Item = &'a PoolUnused>,
    compression_format: CompressionFormat,
) -> Result<()> {
    let missing_path = pool_path.join("missing");

    let mut writer = ProtobufWriter::<CompressedWriter, PoolUnused>::new_compressed(
        &missing_path,
        true,
        compression_format,
    )
    .await?;

    for entry in missing {
        writer.write(entry).await?;
    }
    writer.flush().await?;

    Ok(())
}

/// Outcome of [`check_unused`]: the per-category counts, the loaded (and possibly
/// walk-augmented, for chunks found `in_nothing`) [`Refcnt`], and the set of chunk hashes
/// actually observed on disk during the walk. The latter two are handed to
/// [`check_missing`] so it can determine which refcnt entries are missing without
/// re-touching the disk.
pub struct CheckUnusedOutcome {
    pub count: FsckUnusedCount,
    pub refcnt: Refcnt,
    pub seen: HashSet<[u8; 32]>,
}

/// Identifies unused chunks in the pool by walking the on-disk pool directory and
/// classifying each chunk found against the loaded refcnt.
///
/// This only performs the disk walk (the "unused" phase) — it does not check for refcnt
/// entries that are missing from disk; that's [`check_missing`], which reuses the `seen`
/// set and `Refcnt` this function returns instead of re-walking or re-`stat`-ing anything.
///
/// # Arguments
///
/// * `dry_run` - If true, do not persist the recalculated refcnt.
/// * `progress_tx` - Channel for progress updates.
/// * `config` - Reference to the Woodstock [`Configuration`] struct.
///
/// # Errors
///
/// Returns an error if the pool cannot be loaded or the unused chunk check fails.
pub async fn check_unused(
    dry_run: bool,
    progress_tx: mpsc::Sender<FsckUnusedCount>,
    config: Arc<Configuration>,
    cancel_token: &CancellationToken,
) -> Result<CheckUnusedOutcome> {
    let mut pool_refcnt = Refcnt::new(&config.path.pool_path);
    pool_refcnt.load_refcnt(false).await;

    let mut count = FsckUnusedCount {
        in_unused: 0,
        in_refcnt: 0,
        in_nothing: 0,
    };
    let mut seen: HashSet<[u8; 32]> = HashSet::new();

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
    let mut processed: usize = 0;
    while let Some(hash) = entries.next().await {
        if cancel_token.is_cancelled() {
            cancelled = true;
            break;
        }

        processed += 1;

        // Record presence for check_missing — every hash reaching this point is a real
        // `.zz` file on disk. A malformed 64-hex-char filename (shouldn't happen; chunk
        // filenames are always the hash itself) just doesn't get recorded as seen.
        if let Ok(fixed_hash) = <[u8; 32]>::try_from(hash.as_slice()) {
            seen.insert(fixed_hash);
        }

        let wrapper = PoolChunkWrapper::new(&config.path.pool_path, Some(&hash));
        // SAFETY: wrapper was created with Some(&hash) — get_hash_str() always returns Some
        let hash_str = wrapper.get_hash_str().as_ref().unwrap();

        let refcnt = pool_refcnt.get_refcnt(&hash);
        // Send on every anomaly (in_nothing) for immediate visibility, otherwise batched —
        // this walk can cover millions of chunks and a message per item would flood the
        // progress channel and the Redis snapshot it feeds.
        let mut should_send = processed.is_multiple_of(FSCK_PROGRESS_BATCH_SIZE);
        if refcnt.is_some_and(|r| r.ref_count == 0) {
            count.in_unused += 1;
        } else if refcnt.is_some_and(|r| r.ref_count > 0) {
            count.in_refcnt += 1;
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
            should_send = true;
        }

        if should_send {
            if let Err(e) = progress_tx.send(count.clone()).await {
                error!("Failed to send progress update: {}", e);
            }
        }
    }
    if let Err(e) = progress_tx.send(count.clone()).await {
        error!("Failed to send progress update: {}", e);
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

    Ok(CheckUnusedOutcome {
        count,
        refcnt: pool_refcnt,
        seen,
    })
}

/// Identifies refcnt entries that reference a chunk absent from disk — the counterpart to
/// [`check_unused`], run as its own fsck phase.
///
/// Reuses the `refcnt` and `seen` set produced by [`check_unused`] instead of re-walking or
/// `stat`-ing the pool: a refcnt entry is missing precisely when its hash isn't in `seen`.
/// This avoids one `stat()` syscall per refcnt entry (which, scattered across the pool's
/// 3-level hex-sharded directories, previously dominated fsck runtime on large pools).
///
/// # Errors
///
/// Returns an error if the durable "missing chunks" snapshot cannot be written.
pub async fn check_missing(
    refcnt: &Refcnt,
    seen: &HashSet<[u8; 32]>,
    progress_tx: mpsc::Sender<FsckMissingCount>,
    config: Arc<Configuration>,
    cancel_token: &CancellationToken,
) -> Result<FsckMissingCount> {
    let previously_missing = load_missing_chunks(&config.path.pool_path).await;

    let mut count = FsckMissingCount::default();
    let mut still_missing = Vec::new();
    let mut cancelled = false;

    for entry in refcnt.list_refcnt() {
        if cancel_token.is_cancelled() {
            cancelled = true;
            break;
        }

        count.checked += 1;

        let is_present = <[u8; 32]>::try_from(entry.sha256.as_slice())
            .is_ok_and(|hash| seen.contains(&hash));

        let mut should_send = count.checked.is_multiple_of(FSCK_PROGRESS_BATCH_SIZE);
        if !is_present {
            count.missing += 1;
            should_send = true;
            if previously_missing.contains_key(&entry.sha256) {
                warn!(
                    "{} is missing (refcnt {}) — already known missing since last check",
                    hex::encode(&entry.sha256),
                    entry.ref_count
                );
            } else {
                error!(
                    "{} is missing (refcnt {})",
                    hex::encode(&entry.sha256),
                    entry.ref_count
                );
            }
            still_missing.push(PoolUnused {
                sha256: entry.sha256.clone(),
                size: entry.size,
                compressed_size: entry.compressed_size,
            });
        }

        if should_send {
            if let Err(e) = progress_tx.send(count.clone()).await {
                error!("Failed to send missing chunks verification progress: {}", e);
            }
        }
    }
    if let Err(e) = progress_tx.send(count.clone()).await {
        error!("Failed to send missing chunks verification progress: {}", e);
    }

    // Refresh the durable `missing` snapshot every run (informational, not a mutation of
    // pool state) so the next check can tell a recurring miss from a new one — unless this
    // scan itself was cancelled part-way, in which case the partial `still_missing` list
    // isn't trustworthy enough to persist.
    if !cancelled {
        if let Err(e) = save_missing_chunks(
            &config.path.pool_path,
            still_missing.iter(),
            config.compression_format,
        )
        .await
        {
            error!("Failed to save missing-chunks file: {e}");
        }
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn missing_chunks_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let pool_path = dir.path();

        // No snapshot on disk yet — load must yield an empty map, not an error.
        let loaded = load_missing_chunks(pool_path).await;
        assert!(loaded.is_empty());

        let entries = vec![
            PoolUnused {
                sha256: vec![0x1],
                size: 10,
                compressed_size: 5,
            },
            PoolUnused {
                sha256: vec![0x2],
                size: 20,
                compressed_size: 8,
            },
        ];

        save_missing_chunks(pool_path, entries.iter(), CompressionFormat::Zstd)
            .await
            .unwrap();

        let loaded = load_missing_chunks(pool_path).await;
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded.get(&vec![0x1]).unwrap().size, 10);
        assert_eq!(loaded.get(&vec![0x2]).unwrap().compressed_size, 8);

        // Simulate healing one entry and persisting the shrunk set.
        let remaining = vec![entries[1].clone()];
        save_missing_chunks(pool_path, remaining.iter(), CompressionFormat::Zstd)
            .await
            .unwrap();

        let loaded = load_missing_chunks(pool_path).await;
        assert_eq!(loaded.len(), 1);
        assert!(loaded.contains_key(&vec![0x2]));
    }
}

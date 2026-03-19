use std::{collections::BTreeMap, sync::Arc};

use eyre::Result;
use futures::{pin_mut, StreamExt};
use tokio::fs::read_dir;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::{
    config::{BackupStatus, Backups, Configuration, Hosts},
    pool::{data, IndexedChunk, IndexedSegment, PoolIndex},
};
use uuid::Uuid;

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
/// - [`PoolChunkWrapper`]: For related pool operations
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

#[derive(Default)]
struct ScannedPoolV3 {
    chunks: BTreeMap<Vec<u8>, IndexedChunk>,
    segments: BTreeMap<u64, IndexedSegment>,
    duplicate_count: usize,
}

async fn scan_pool_v3_segments(config: &Configuration) -> Result<ScannedPoolV3> {
    let segments_path = data::segments_directory_path(&config.path.pool_path);
    if !segments_path.exists() {
        return Ok(ScannedPoolV3::default());
    }

    let mut scanned = ScannedPoolV3::default();
    let mut entries = read_dir(&segments_path).await?;
    let mut segment_paths = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().is_some_and(|extension| extension == "seg") {
            segment_paths.push(path);
        }
    }

    segment_paths.sort();

    for path in segment_paths {
        let segment_file = data::open_existing_segment(&path).await?;
        let chunk_entries = segment_file.chunks().await?;
        let segment_id = segment_file.header().segment_id;

        for chunk_entry in &chunk_entries {
            let hash = chunk_entry.hash.clone();
            if scanned.chunks.contains_key(&hash) {
                scanned.duplicate_count += 1;
                error!(
                    "Pool V3 duplicate physical chunk {} found in segment {}",
                    hex::encode(&hash),
                    segment_id
                );
                continue;
            }

            scanned.chunks.insert(
                hash.clone(),
                IndexedChunk {
                    hash,
                    size: chunk_entry.size,
                    compressed_size: chunk_entry.compressed_size,
                    compression_format: chunk_entry.compression_format,
                    ref_count: 0,
                    segment_id,
                    offset: chunk_entry.header_offset,
                    chunk_header_size: chunk_entry.chunk_header_size,
                },
            );
        }

        scanned.segments.insert(
            segment_id,
            IndexedSegment {
                segment_id,
                state: segment_file.state(),
                size_total: segment_file.size_total(),
                size_effective: 0,
                size_limit: segment_file.header().target_size,
                chunk_count: chunk_entries.len() as u64,
            },
        );
    }

    Ok(scanned)
}

async fn collect_backup_refcounts(
    backups: &Backups,
    hostname: &str,
    backup_id: Uuid,
) -> Result<BTreeMap<Vec<u8>, u64>> {
    let mut refcounts = BTreeMap::new();

    for manifest in backups.get_manifests(hostname, backup_id).await {
        let chunks = manifest.list_chunks();
        pin_mut!(chunks);

        while let Some(chunk) = chunks.next().await {
            if chunk.chunk_hash.is_empty() {
                continue;
            }

            *refcounts.entry(chunk.chunk_hash).or_insert(0) += 1;
        }
    }

    Ok(refcounts)
}

async fn collect_host_refcounts(
    backups: &Backups,
    hostname: &str,
) -> Result<BTreeMap<Vec<u8>, u64>> {
    let mut refcounts = BTreeMap::new();

    for backup in backups.get_backups(hostname).await {
        if backup.status != BackupStatus::Completed {
            continue;
        }

        for (hash, count) in collect_backup_refcounts(backups, hostname, backup.id).await? {
            *refcounts.entry(hash).or_insert(0) += count;
        }
    }

    Ok(refcounts)
}

async fn collect_pool_refcounts(
    hosts_config: &Hosts,
    backups: &Backups,
) -> Result<BTreeMap<Vec<u8>, u64>> {
    let mut refcounts = BTreeMap::new();

    for hostname in hosts_config.list_hosts().await? {
        for (hash, count) in collect_host_refcounts(backups, &hostname).await? {
            *refcounts.entry(hash).or_insert(0) += count;
        }
    }

    Ok(refcounts)
}

fn compare_expected_visibility(
    index: Option<&PoolIndex>,
    refcounts: &BTreeMap<Vec<u8>, u64>,
) -> Result<usize> {
    let Some(index) = index else {
        return Ok(refcounts.len());
    };

    let mut error_count = 0;
    for hash in refcounts.keys() {
        if index.get_chunk(hash)?.is_none() {
            error_count += 1;
            error!(
                "Pool V3 chunk {} referenced by manifest but missing from visible index",
                hex::encode(hash)
            );
        }
    }

    Ok(error_count)
}

fn build_rebuilt_pool_v3_state(
    scanned: ScannedPoolV3,
    expected_refcounts: BTreeMap<Vec<u8>, u64>,
) -> (
    BTreeMap<Vec<u8>, IndexedChunk>,
    BTreeMap<u64, IndexedSegment>,
    usize,
) {
    let mut chunks = scanned.chunks;
    let mut segments = scanned.segments;
    let mut error_count = scanned.duplicate_count;

    for (hash, ref_count) in expected_refcounts {
        let Some(chunk) = chunks.get_mut(&hash) else {
            error_count += 1;
            error!(
                "Pool V3 chunk {} referenced by manifests but missing from segments",
                hex::encode(&hash)
            );
            continue;
        };

        chunk.ref_count = ref_count;
        let Some(segment) = segments.get_mut(&chunk.segment_id) else {
            error_count += 1;
            error!(
                "Pool V3 segment {} missing from rebuilt state for chunk {}",
                chunk.segment_id,
                hex::encode(&hash)
            );
            continue;
        };

        segment.size_effective = segment
            .size_effective
            .saturating_add(chunk.chunk_header_size + chunk.compressed_size);
    }

    (chunks, segments, error_count)
}

fn compare_chunk_maps(
    current: &BTreeMap<Vec<u8>, IndexedChunk>,
    rebuilt: &BTreeMap<Vec<u8>, IndexedChunk>,
) -> usize {
    let mut error_count = 0;

    for (hash, rebuilt_chunk) in rebuilt {
        match current.get(hash) {
            Some(current_chunk) if current_chunk == rebuilt_chunk => {}
            Some(current_chunk) => {
                error_count += 1;
                error!(
                    "Pool V3 chunk {} mismatch: current {:?}, rebuilt {:?}",
                    hex::encode(hash),
                    current_chunk,
                    rebuilt_chunk
                );
            }
            None => {
                error_count += 1;
                error!("Pool V3 chunk {} missing from index", hex::encode(hash));
            }
        }
    }

    for hash in current.keys() {
        if !rebuilt.contains_key(hash) {
            error_count += 1;
            error!(
                "Pool V3 stale chunk {} present only in index",
                hex::encode(hash)
            );
        }
    }

    error_count
}

fn compare_segment_maps(
    current: &BTreeMap<u64, IndexedSegment>,
    rebuilt: &BTreeMap<u64, IndexedSegment>,
) -> usize {
    let mut error_count = 0;

    for (segment_id, rebuilt_segment) in rebuilt {
        match current.get(segment_id) {
            Some(current_segment) if current_segment == rebuilt_segment => {}
            Some(current_segment) => {
                error_count += 1;
                error!(
                    "Pool V3 segment {} mismatch: current {:?}, rebuilt {:?}",
                    segment_id, current_segment, rebuilt_segment
                );
            }
            None => {
                error_count += 1;
                error!("Pool V3 segment {} missing from index", segment_id);
            }
        }
    }

    for segment_id in current.keys() {
        if !rebuilt.contains_key(segment_id) {
            error_count += 1;
            error!("Pool V3 stale segment {} present only in index", segment_id);
        }
    }

    error_count
}

fn reconcile_pool_v3_index(
    index: &PoolIndex,
    current_chunks: &BTreeMap<Vec<u8>, IndexedChunk>,
    rebuilt_chunks: &BTreeMap<Vec<u8>, IndexedChunk>,
    current_segments: &BTreeMap<u64, IndexedSegment>,
    rebuilt_segments: &BTreeMap<u64, IndexedSegment>,
) -> Result<()> {
    for hash in current_chunks.keys() {
        if !rebuilt_chunks.contains_key(hash) {
            index.remove_chunk(hash)?;
        }
    }

    for chunk in rebuilt_chunks.values() {
        index.add_chunk(chunk)?;
    }

    for segment_id in current_segments.keys() {
        if !rebuilt_segments.contains_key(segment_id) {
            index.remove_segment(*segment_id)?;
        }
    }

    for segment in rebuilt_segments.values() {
        index.add_segment(segment)?;
    }

    Ok(())
}

/// Checks the integrity of the reference counts.
///
/// # Arguments
/// * `original_refcnt` - The original reference count data.
/// * `new_refcnt` - The new reference count data to compare against.
///
/// # Returns
/// The number of errors found during the integrity check.

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
    let Some(backup) = backups.get_backup(hostname, backup_id).await else {
        warn!("Pool V3 fsck skipped missing backup {hostname}/{backup_id}");
        return Ok(FsckCount {
            error_count: 1,
            total_count: 0,
        });
    };

    if backup.status != BackupStatus::Completed {
        return Ok(FsckCount {
            error_count: 0,
            total_count: 0,
        });
    }

    let index_path = data::pool_index_path(&config.path.pool_path);
    let index = if index_path.exists() {
        Some(data::open_pool_index(&config.path.pool_path)?)
    } else {
        None
    };
    let refcounts = collect_backup_refcounts(&backups, hostname, backup_id).await?;
    let error_count = compare_expected_visibility(index.as_ref(), &refcounts)?;

    let _ = dry_run;
    Ok(FsckCount {
        error_count,
        total_count: refcounts.len(),
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
    let index_path = data::pool_index_path(&config.path.pool_path);
    let index = if index_path.exists() {
        Some(data::open_pool_index(&config.path.pool_path)?)
    } else {
        None
    };
    let refcounts = collect_host_refcounts(&backups, hostname).await?;
    let error_count = compare_expected_visibility(index.as_ref(), &refcounts)?;

    let _ = dry_run;
    Ok(FsckCount {
        error_count,
        total_count: refcounts.len(),
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
    let scanned = scan_pool_v3_segments(&config).await?;
    let expected_refcounts = collect_pool_refcounts(&hosts_config, &backups_config).await?;
    let total_count = scanned.chunks.len();
    let (rebuilt_chunks, rebuilt_segments, mut error_count) =
        build_rebuilt_pool_v3_state(scanned, expected_refcounts);

    let index_path = data::pool_index_path(&config.path.pool_path);
    let (current_chunks, current_segments) = if index_path.exists() {
        let index = data::open_pool_index(&config.path.pool_path)?;
        (
            index
                .list_all_chunks()?
                .into_iter()
                .map(|chunk| (chunk.hash.clone(), chunk))
                .collect::<BTreeMap<_, _>>(),
            index
                .list_segments()?
                .into_iter()
                .map(|segment| (segment.segment_id, segment))
                .collect::<BTreeMap<_, _>>(),
        )
    } else {
        if !rebuilt_chunks.is_empty() || !rebuilt_segments.is_empty() {
            error_count += 1;
            error!("Pool V3 index is missing while segments exist");
        }
        (BTreeMap::new(), BTreeMap::new())
    };

    error_count += compare_chunk_maps(&current_chunks, &rebuilt_chunks);
    error_count += compare_segment_maps(&current_segments, &rebuilt_segments);

    if !dry_run && error_count > 0 {
        info!("Repair Pool V3 main index from segments and manifests");
        let index = data::open_pool_index(&config.path.pool_path)?;
        reconcile_pool_v3_index(
            &index,
            &current_chunks,
            &rebuilt_chunks,
            &current_segments,
            &rebuilt_segments,
        )?;
    }

    Ok(FsckCount {
        error_count,
        total_count,
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
) -> Result<FsckUnusedCount> {
    let scanned = scan_pool_v3_segments(&config).await?;
    let index_path = data::pool_index_path(&config.path.pool_path);
    let index = if index_path.exists() {
        Some(data::open_pool_index(&config.path.pool_path)?)
    } else {
        None
    };

    let current_chunks = index
        .as_ref()
        .map(|pool_index| {
            pool_index.list_all_chunks().map(|chunks| {
                chunks
                    .into_iter()
                    .map(|chunk| (chunk.hash.clone(), chunk))
                    .collect::<BTreeMap<_, _>>()
            })
        })
        .transpose()?
        .unwrap_or_default();

    let mut count = FsckUnusedCount {
        in_unused: 0,
        in_refcnt: 0,
        in_nothing: 0,
        missing: 0,
    };

    for hash in scanned.chunks.keys() {
        match current_chunks.get(hash) {
            Some(chunk) if chunk.ref_count == 0 => count.in_unused += 1,
            Some(_) => count.in_refcnt += 1,
            None => {
                count.in_nothing += 1;
                error!(
                    "Pool V3 physical chunk {} is not indexed",
                    hex::encode(hash)
                );
            }
        }

        if let Err(send_error) = progress_tx.send(count.clone()).await {
            error!("Failed to send progress update: {}", send_error);
        }
    }

    for hash in current_chunks.keys() {
        if !scanned.chunks.contains_key(hash) {
            count.missing += 1;
            error!(
                "Pool V3 indexed chunk {} is missing from segments",
                hex::encode(hash)
            );
        }
    }

    let _ = dry_run;
    Ok(count)
}

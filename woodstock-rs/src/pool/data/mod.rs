//! Pool V3 low-level storage access layer.
//!
//! This module centralizes the on-disk layout of the unified Pool V3 index and append-only
//! segments. High-level pool components should resolve paths, open existing segments, and create
//! new nominal segments through this layer instead of rebuilding the layout locally.
//!
//! Segment files are self-describing and do not rely on any sidecar metadata file.

use std::path::{Path, PathBuf};

use eyre::Result;

pub mod internal;
pub mod segments;

pub use internal::index::{IndexedChunk, IndexedSegment, PoolIndex};
pub use internal::segment::{
    SegmentChunkEntry, SegmentChunkReader, SegmentFile, SegmentFileHeader, SegmentFileState,
    SEGMENT_FORMAT_VERSION,
};
pub use segments::{
    cleanup_compacted_segment, cleanup_unpublished_compaction_targets,
    compaction_temp_segment_path, create_and_reserve_segment, create_compaction_target,
    reserve_open_or_create_segment, segment_compaction_lock_resource, segment_write_lock_resource,
    try_reserve_existing_segment, try_reserve_segment_compaction_lock, CompactionTarget,
    SegmentReservation,
};

const SEGMENTS_DIRECTORY_NAME: &str = "segments";
const INDEX_DIRECTORY_NAME: &str = "index";
const PENDING_DIRECTORY_NAME: &str = "pending";

pub struct CreatedPoolSegment {
    pub segment: IndexedSegment,
    pub path: PathBuf,
    pub file: SegmentFile,
}

#[must_use]
pub fn pool_index_path(pool_path: &Path) -> PathBuf {
    index_directory_path(pool_path)
}

pub fn open_pool_index(pool_path: &Path) -> Result<PoolIndex> {
    PoolIndex::open_or_create(pool_index_path(pool_path))
}

#[must_use]
pub fn segments_directory_path(pool_path: &Path) -> PathBuf {
    pool_path.join(SEGMENTS_DIRECTORY_NAME)
}

#[must_use]
pub fn segment_relative_path(segment_id: u64) -> String {
    format!("{SEGMENTS_DIRECTORY_NAME}/seg-{segment_id:011}.seg")
}

#[must_use]
pub fn segment_path(pool_path: &Path, segment_id: u64) -> PathBuf {
    pool_path.join(segment_relative_path(segment_id))
}

#[must_use]
pub fn resolve_segment_path(pool_path: &Path, segment: &IndexedSegment) -> PathBuf {
    segment_path(pool_path, segment.segment_id)
}

#[must_use]
pub fn index_directory_path(pool_path: &Path) -> PathBuf {
    pool_path.join(INDEX_DIRECTORY_NAME)
}

#[must_use]
pub fn pending_directory_path(pool_path: &Path) -> PathBuf {
    index_directory_path(pool_path).join(PENDING_DIRECTORY_NAME)
}

pub async fn open_existing_segment(path: &Path) -> Result<SegmentFile> {
    SegmentFile::open(path).await
}

pub async fn open_indexed_segment(
    pool_path: &Path,
    segment: &IndexedSegment,
) -> Result<SegmentFile> {
    open_existing_segment(&resolve_segment_path(pool_path, segment)).await
}

pub async fn create_segment(
    pool_path: &Path,
    segment_id: u64,
    target_size: u64,
) -> Result<CreatedPoolSegment> {
    let path = segment_path(pool_path, segment_id);
    let file = SegmentFile::create(&path, segment_id, target_size).await?;
    let segment = IndexedSegment {
        segment_id,
        state: file.state(),
        size_total: file.size_total(),
        size_effective: 0,
        size_limit: target_size,
        chunk_count: 0,
    };

    Ok(CreatedPoolSegment {
        segment,
        path,
        file,
    })
}

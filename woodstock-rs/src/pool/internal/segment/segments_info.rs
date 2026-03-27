//! Persistent metadata for the segment directory: `segments/segments.info`.
//!
//! This file acts as a low-cost cache that lets [`super::segments::Segments`]
//! skip a full directory scan on every `get_segment_writer()` call.  It stores:
//!
//! - the ID of the oldest still-open segment ([`SegmentsInformation::first_open_segment_id`]),
//! - the ID to assign to the next new segment ([`SegmentsInformation::next_segment_id`]),
//! - a monotonic [`SegmentsInformation::generation`] counter incremented on each write.
//!
//! Writes are **atomic**: data is first flushed to a uniquely-named temporary file
//! inside the same directory, then renamed over the target — a single kernel call
//! that is crash-safe on any POSIX filesystem.
//!
//! The caller is responsible for holding an exclusive Redis lock
//! (`LockOperation::Segment(SegmentLockTarget::Info)`) for the entire
//! read-modify-write cycle.

use std::path::{Path, PathBuf};

use eyre::Result;
use tokio::fs::File;
use tokio::io::AsyncWriteExt as _;
use uuid::Uuid;

use super::segment_protobuf::SegmentsInformation;
use crate::proto::{read_length_delimited_message, write_length_delimited_message};

/// Returns the canonical path of the metadata file for the given segment directory.
pub(crate) fn segments_info_path(dir: &Path) -> PathBuf {
    dir.join("segments.info")
}

/// Reads [`SegmentsInformation`] from `segments.info` inside `dir`.
///
/// Returns `Ok(None)` when the file does not exist (first run or legacy directory
/// without a metadata file — the caller should fall back to `list_segments()`).
///
/// # Errors
///
/// Returns an error if the file exists but cannot be parsed.
pub(crate) async fn read_segments_info(dir: &Path) -> Result<Option<SegmentsInformation>> {
    let path = segments_info_path(dir);

    if !path.exists() {
        return Ok(None);
    }

    let file = File::open(&path).await?;
    let mut reader = tokio::io::BufReader::new(file);
    let Some((record, _)) =
        read_length_delimited_message::<_, SegmentsInformation>(&mut reader).await?
    else {
        return Ok(None);
    };

    Ok(Some(record))
}

/// Atomically persists `info` to `segments.info` inside `dir`.
///
/// The data is first written to a uniquely-named temporary file (same directory,
/// so the rename stays on the same filesystem), then atomically renamed to the
/// final path.
///
/// # Errors
///
/// Returns an error if the file cannot be created, written, or renamed.
pub(crate) async fn write_segments_info_atomic(
    dir: &Path,
    info: &SegmentsInformation,
) -> Result<()> {
    let final_path = segments_info_path(dir);
    let tmp_name = format!(".segments.info.{}", Uuid::new_v4());
    let tmp_path = dir.join(tmp_name);

    let mut file = File::create(&tmp_path).await?;
    write_length_delimited_message(&mut file, info).await?;
    file.flush().await?;
    file.shutdown().await?;
    drop(file);

    tokio::fs::rename(&tmp_path, &final_path).await?;

    Ok(())
}

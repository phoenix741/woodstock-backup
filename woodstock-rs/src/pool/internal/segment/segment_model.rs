//! Shared append-only segment file primitives for Pool V3.
//!
//! This module defines the on-disk format and common parsing helpers used by
//! [`super::segment_reader::SegmentReader`] and [`super::segment_writer::SegmentWriter`].
//!
//! The segment is intentionally kept simple:
//! - a single segment header at the start of the file;
//! - a linear sequence of chunk headers followed by chunk payloads;
//! - no local index, footer, checksum table, or staging logic in the segment itself.
//!
//! The file layout is:
//! 1. one length-delimited [`crate::SegmentHeader`];
//! 2. zero or more length-delimited [`crate::SegmentChunkHeader`] values;
//! 3. after each chunk header, exactly `compressed_size` bytes of payload.

use super::segment_protobuf::{SegmentFileMetadataRecord, SegmentHeader};
use crate::utils::compression::CompressionFormat;

/// Current on-disk format version used for Pool V3 segment files.
pub const SEGMENT_FORMAT_VERSION: u32 = 1;

/// Logical openness state of a segment file.
///
/// The state is derived from the current file size relative to the configured
/// target size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentFileState {
    /// The segment can still accept more appended chunks.
    Open,
    /// The segment reached or exceeded its configured target size.
    Full,
}

impl SegmentFileState {
    #[must_use]
    pub fn as_u32(self) -> u32 {
        match self {
            Self::Open => 0,
            Self::Full => 1,
        }
    }
}

impl TryFrom<u32> for SegmentFileState {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Open),
            1 => Ok(Self::Full),
            _ => Err("invalid segment file state"),
        }
    }
}

/// Header stored at the beginning of every segment file.
///
/// This record is immutable once the segment has been created.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentFileHeader {
    /// Segment format version written on disk.
    pub format_version: u32,
    /// Monotonic identifier assigned by the pool index.
    pub segment_id: u64,
    /// Target segment size used to determine when the segment becomes full.
    pub target_size: u64,
    /// Creation timestamp in Unix seconds.
    pub created_at: u64,
}

/// A segment file metadata as stored in the pool index.
///
/// Each `SegmentFileMetadata` maps a numeric identifier to the corresponding `.seg` file and
/// carries enough metadata to make scheduling decisions (e.g. whether to open a new segment
/// or continue appending to an existing one).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentFileMetadata {
    /// Monotonically increasing identifier that deterministically maps to a file path under
    /// `pool/segments/<id>.seg`.
    pub segment_id: u64,
    /// Lifecycle state of the segment: `Open` accepts new chunks, `Full` is sealed.
    pub state: SegmentFileState,
    /// Physical file size in bytes (total bytes written including headers).
    pub size_total: u64,
    /// Sum of uncompressed chunk sizes for all chunks in this segment.
    pub size_effective: u64,
    /// Maximum allowed `size_total` before the segment is marked `Full`.
    pub size_limit: u64,
    /// Number of chunk entries recorded in this segment.
    pub chunk_count: u64,
}

impl From<&SegmentFileMetadata> for SegmentFileMetadataRecord {
    fn from(value: &SegmentFileMetadata) -> Self {
        Self {
            segment_id: value.segment_id,
            state: value.state.as_u32(),
            size_total: value.size_total,
            size_effective: value.size_effective,
            size_limit: value.size_limit,
            chunk_count: value.chunk_count,
        }
    }
}

impl TryFrom<SegmentFileMetadataRecord> for SegmentFileMetadata {
    type Error = &'static str;

    fn try_from(value: SegmentFileMetadataRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            segment_id: value.segment_id,
            state: SegmentFileState::try_from(value.state)?,
            size_total: value.size_total,
            size_effective: value.size_effective,
            size_limit: value.size_limit,
            chunk_count: value.chunk_count,
        })
    }
}

/// Physical location of one stored chunk inside a segment file.
///
/// The entry contains both logical metadata copied from the chunk header and the
/// byte offsets required to reopen the payload later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentChunkEntry {
    /// Chunk hash used by the pool index as the logical identifier.
    pub hash: Vec<u8>,
    /// Uncompressed chunk size in bytes.
    pub size: u64,
    /// Stored compressed payload size in bytes.
    pub compressed_size: u64,
    /// Compression format of the stored payload.
    pub compression_format: CompressionFormat,
    /// Offset of the chunk header within the segment file.
    pub header_offset: u64,
    /// Offset of the chunk payload within the segment file.
    pub payload_offset: u64,
    /// Serialized length-delimited chunk header size in bytes.
    pub chunk_header_size: u64,
}

impl SegmentChunkEntry {
    /// Returns the total number of bytes occupied by this chunk entry on disk.
    #[must_use]
    pub fn stored_len(&self) -> u64 {
        self.chunk_header_size + self.compressed_size
    }
}

impl From<SegmentHeader> for SegmentFileHeader {
    fn from(value: SegmentHeader) -> Self {
        Self {
            format_version: value.format_version,
            segment_id: value.segment_id,
            target_size: value.target_size,
            created_at: value.created_at,
        }
    }
}

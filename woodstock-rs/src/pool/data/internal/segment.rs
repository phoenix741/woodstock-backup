//! Append-only segment file primitives for Pool V3.
//!
//! This module implements a minimal on-disk segment format used to store already-compressed
//! chunks sequentially inside a single file.
//!
//! The segment is intentionally kept simple:
//! - a single segment header at the start of the file;
//! - a linear sequence of chunk headers followed by chunk payloads;
//! - no local index, footer, checksum table, or staging logic in the segment itself.
//!
//! The caller remains responsible for producing a chunk payload beforehand, including any
//! hashing, compression, or temporary-file lifecycle management. [`SegmentFile`] only appends a
//! chunk that already exists on disk and records the metadata needed to locate it later.
//!
//! The file layout is:
//! 1. one length-delimited [`crate::PoolV3SegmentHeader`];
//! 2. zero or more length-delimited [`crate::PoolV3ChunkHeader`] values;
//! 3. after each chunk header, exactly `compressed_size` bytes of payload.
//!
//! Public APIs in this module allow:
//! - creating and reopening segment files;
//! - listing chunk entries discovered in a segment;
//! - opening a bounded reader for a single stored chunk;
//! - appending a chunk from an existing source path;
//! - querying whether the segment has reached its configured capacity.
//!
//! The segment file is self-describing and remains readable without any auxiliary
//! metadata file. Callers that need exact counters after a reopen must rescan the
//! segment through [`Self::chunks`] instead of relying on an external sidecar.

use std::io::{self, ErrorKind, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use eyre::{bail, eyre, Result};
use prost::Message;
use tokio::fs::{create_dir_all, metadata, File, OpenOptions};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufReader, BufWriter};

use crate::pool::{PoolV3ChunkHeader, PoolV3SegmentHeader};
use crate::proto::{
    read_length_delimited_message, read_optional_length_delimited_message,
    write_length_delimited_message,
};
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

/// Reader limited to the exact compressed payload length of one chunk entry.
pub type SegmentChunkReader = tokio::io::Take<File>;

/// Append-only segment file abstraction.
///
/// A segment file owns one immutable header and a linear sequence of chunk
/// entries appended over time. The type provides low-level storage primitives
/// only; logical visibility and reference counting are handled by the pool index.
#[derive(Debug)]
pub struct SegmentFile {
    path: PathBuf,
    header: SegmentFileHeader,
    size_total: u64,
    data_offset: u64,
}

impl SegmentFile {
    /// Creates a fresh segment file using the current wall clock as `created_at`.
    pub async fn create<P: AsRef<Path>>(
        path: P,
        segment_id: u64,
        target_size: u64,
    ) -> Result<Self> {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| eyre!(error))?
            .as_secs();

        Self::create_with_created_at(path, segment_id, target_size, created_at).await
    }

    /// Creates a fresh segment file with an explicit creation timestamp.
    ///
    /// This is primarily useful for deterministic tests and metadata replay.
    pub async fn create_with_created_at<P: AsRef<Path>>(
        path: P,
        segment_id: u64,
        target_size: u64,
        created_at: u64,
    ) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            create_dir_all(parent).await?;
        }

        let header_record = PoolV3SegmentHeader {
            format_version: SEGMENT_FORMAT_VERSION,
            segment_id,
            target_size,
            created_at,
        };
        let file = File::create(&path).await?;
        let mut writer = BufWriter::new(file);
        let header_size =
            u64::try_from(write_length_delimited_message(&mut writer, &header_record).await?)?;
        writer.flush().await?;
        writer.shutdown().await?;

        let segment = Self {
            path,
            header: header_record.into(),
            size_total: header_size,
            data_offset: header_size,
        };

        Ok(segment)
    }

    /// Opens an existing segment file.
    ///
    /// Reopening a segment only parses the immutable segment header and records
    /// the current physical file size. Exact chunk counters are intentionally not
    /// reconstructed here; callers that need them must rescan the segment through
    /// [`Self::chunks`].
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path).await?;
        let mut file = BufReader::new(file);
        let mut protobuf_buffer = Vec::with_capacity(128);
        let (header_record, header_size) = read_length_delimited_message::<PoolV3SegmentHeader, _>(
            &mut file,
            &mut protobuf_buffer,
        )
        .await?;
        let file_size = metadata(&path).await?.len();
        let data_offset = u64::try_from(header_size)?;

        Ok(Self {
            path,
            header: header_record.into(),
            size_total: file_size,
            data_offset,
        })
    }

    /// Returns the filesystem path of the segment file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the immutable segment header loaded from disk.
    #[must_use]
    pub fn header(&self) -> &SegmentFileHeader {
        &self.header
    }

    /// Returns the total physical size of the segment file in bytes.
    #[must_use]
    pub fn size_total(&self) -> u64 {
        self.size_total
    }

    /// Returns the byte offset where chunk entries start.
    #[must_use]
    pub fn data_offset(&self) -> u64 {
        self.data_offset
    }

    /// Returns whether the segment has reached its target size.
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.size_total >= self.header.target_size
    }

    /// Returns the derived current state of the segment.
    #[must_use]
    pub fn state(&self) -> SegmentFileState {
        if self.is_full() {
            SegmentFileState::Full
        } else {
            SegmentFileState::Open
        }
    }

    /// Returns how many more bytes can still be appended before the target size.
    #[must_use]
    pub fn remaining_capacity(&self) -> u64 {
        self.header.target_size.saturating_sub(self.size_total)
    }

    /// Scans the segment file and returns every complete chunk entry discovered on disk.
    ///
    /// If the file ends with a truncated header or payload, scanning stops at the
    /// last complete chunk instead of failing hard.
    pub async fn chunks(&self) -> Result<Vec<SegmentChunkEntry>> {
        let file = File::open(&self.path).await?;
        let mut file = BufReader::new(file);
        let file_size = metadata(&self.path).await?.len();
        file.seek(SeekFrom::Start(self.data_offset)).await?;

        let mut chunks = Vec::new();
        let mut protobuf_buffer = Vec::with_capacity(256);

        loop {
            let header_offset = file.stream_position().await?;

            let (chunk_header, chunk_header_size) =
                match read_optional_length_delimited_message::<PoolV3ChunkHeader, _>(
                    &mut file,
                    &mut protobuf_buffer,
                )
                .await?
                {
                    Some(result) => result,
                    None => break,
                };
            let chunk_header_size = u64::try_from(chunk_header_size)?;

            let payload_offset = file.stream_position().await?;
            let next_chunk_offset = match payload_offset.checked_add(chunk_header.compressed_size) {
                Some(offset) => offset,
                None => bail!("segment chunk payload offset overflow"),
            };

            if next_chunk_offset > file_size {
                break;
            }

            file.seek(SeekFrom::Start(next_chunk_offset)).await?;

            chunks.push(SegmentChunkEntry {
                hash: chunk_header.hash,
                size: chunk_header.size,
                compressed_size: chunk_header.compressed_size,
                compression_format: CompressionFormat::try_from(chunk_header.compression_format)?,
                header_offset,
                payload_offset,
                chunk_header_size,
            });
        }

        Ok(chunks)
    }

    /// Opens a bounded reader for the chunk whose header starts at `header_offset`.
    ///
    /// The returned reader is limited to the chunk payload size declared in the
    /// header and fails if the payload is truncated on disk.
    pub async fn read_chunk_at(
        &self,
        header_offset: u64,
    ) -> Result<(SegmentChunkEntry, SegmentChunkReader)> {
        let mut file = File::open(&self.path).await?;
        file.seek(SeekFrom::Start(header_offset)).await?;
        let mut protobuf_buffer = Vec::with_capacity(256);

        let (chunk_header, chunk_header_size) =
            read_length_delimited_message::<PoolV3ChunkHeader, _>(&mut file, &mut protobuf_buffer)
                .await?;
        let chunk_header_size = u64::try_from(chunk_header_size)?;
        let payload_offset = file.stream_position().await?;
        let required_len = payload_offset
            .checked_add(chunk_header.compressed_size)
            .ok_or_else(|| eyre!("segment chunk payload offset overflow"))?;

        if metadata(&self.path).await?.len() < required_len {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "segment chunk payload is truncated",
            )
            .into());
        }

        let entry = SegmentChunkEntry {
            hash: chunk_header.hash,
            size: chunk_header.size,
            compressed_size: chunk_header.compressed_size,
            compression_format: CompressionFormat::try_from(chunk_header.compression_format)?,
            header_offset,
            payload_offset,
            chunk_header_size,
        };

        Ok((entry.clone(), file.take(entry.compressed_size)))
    }

    /// Opens a bounded reader for an already known chunk entry.
    pub async fn chunk_reader(&self, entry: &SegmentChunkEntry) -> Result<SegmentChunkReader> {
        let mut file = File::open(&self.path).await?;
        file.seek(SeekFrom::Start(entry.payload_offset)).await?;
        Ok(file.take(entry.compressed_size))
    }

    /// Appends a chunk payload already stored in a source file.
    ///
    /// The source file must contain exactly `compressed_size` bytes.
    pub async fn append_chunk_from_path(
        &mut self,
        hash: Vec<u8>,
        size: u64,
        compressed_size: u64,
        compression_format: CompressionFormat,
        source_path: &Path,
    ) -> Result<SegmentChunkEntry> {
        let source_size = metadata(source_path).await?.len();
        if source_size != compressed_size {
            bail!(
                "segment source size mismatch: expected {compressed_size} bytes, found {source_size} bytes"
            );
        }

        let mut source_file = File::open(source_path).await?;
        self.append_chunk_from_reader(
            hash,
            size,
            compressed_size,
            compression_format,
            &mut source_file,
        )
        .await
    }

    /// Appends a chunk payload from an arbitrary async reader.
    ///
    /// The reader must yield exactly `compressed_size` bytes; otherwise the append
    /// is rejected and the segment reports an error.
    pub async fn append_chunk_from_reader<R>(
        &mut self,
        hash: Vec<u8>,
        size: u64,
        compressed_size: u64,
        compression_format: CompressionFormat,
        reader: &mut R,
    ) -> Result<SegmentChunkEntry>
    where
        R: AsyncRead + Unpin,
    {
        let chunk_header = PoolV3ChunkHeader {
            hash,
            size,
            compressed_size,
            compression_format: compression_format.as_u32(),
        };
        let chunk_header_size = chunk_header.encoded_len();
        let chunk_header_size =
            u64::try_from(prost::length_delimiter_len(chunk_header_size) + chunk_header_size)?;
        let header_offset = self.size_total;
        let payload_offset = header_offset
            .checked_add(chunk_header_size)
            .ok_or_else(|| eyre!("segment chunk payload offset overflow"))?;

        let mut segment_file = OpenOptions::new().append(true).open(&self.path).await?;
        let written_header_size =
            write_length_delimited_message(&mut segment_file, &chunk_header).await?;
        let written_header_size = u64::try_from(written_header_size)?;
        let copied_size = tokio::io::copy(reader, &mut segment_file).await?;
        if copied_size != compressed_size {
            bail!(
                "segment reader size mismatch: expected {compressed_size} bytes, copied {copied_size} bytes"
            );
        }
        segment_file.flush().await?;
        segment_file.shutdown().await?;

        self.size_total = self
            .size_total
            .checked_add(written_header_size)
            .and_then(|value| value.checked_add(compressed_size))
            .ok_or_else(|| eyre!("segment size overflow"))?;

        Ok(SegmentChunkEntry {
            hash: chunk_header.hash,
            size: chunk_header.size,
            compressed_size,
            compression_format,
            header_offset,
            payload_offset,
            chunk_header_size: written_header_size,
        })
    }

    /// Counts complete chunk entries by scanning the segment file directly.
    pub async fn scan_chunk_count<P: AsRef<Path>>(path: P) -> Result<u64> {
        let segment = Self::open(path).await?;
        u64::try_from(segment.chunks().await?.len()).map_err(Into::into)
    }

    pub(crate) fn relocate(&mut self, path: PathBuf) {
        self.path = path;
    }
}

impl From<PoolV3SegmentHeader> for SegmentFileHeader {
    fn from(value: PoolV3SegmentHeader) -> Self {
        Self {
            format_version: value.format_version,
            segment_id: value.segment_id,
            target_size: value.target_size,
            created_at: value.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use tokio::fs::{File, OpenOptions};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;

    async fn create_source_file(path: &Path, payload: &[u8]) {
        let mut file = File::create(path).await.unwrap();
        file.write_all(payload).await.unwrap();
        file.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn create_and_reopen_segment() {
        let tempdir = tempdir().unwrap();
        let segment_path = tempdir.path().join("seg-00000000001.seg");

        let segment = SegmentFile::create_with_created_at(&segment_path, 1, 1024, 1234)
            .await
            .unwrap();

        assert_eq!(segment.header().segment_id, 1);
        assert_eq!(segment.header().target_size, 1024);
        assert_eq!(segment.header().created_at, 1234);
        assert_eq!(segment.state(), SegmentFileState::Open);
        assert_eq!(segment.size_total(), segment.data_offset());

        let reopened = SegmentFile::open(&segment_path).await.unwrap();

        assert_eq!(reopened.header(), segment.header());
        assert_eq!(reopened.size_total(), segment.size_total());
        assert_eq!(reopened.data_offset(), segment.data_offset());
    }

    #[tokio::test]
    async fn append_and_list_chunks() {
        let tempdir = tempdir().unwrap();
        let segment_path = tempdir.path().join("seg-00000000002.seg");
        let first_source_path = tempdir.path().join("first-source.chunk");
        let second_source_path = tempdir.path().join("second-source.chunk");

        let mut segment = SegmentFile::create_with_created_at(&segment_path, 2, 4096, 55)
            .await
            .unwrap();

        create_source_file(&first_source_path, b"first-compressed-payload").await;
        let first_entry = segment
            .append_chunk_from_path(
                vec![0xAA; 32],
                512,
                24,
                CompressionFormat::Zstd,
                &first_source_path,
            )
            .await
            .unwrap();

        create_source_file(&second_source_path, b"raw").await;
        let second_entry = segment
            .append_chunk_from_path(
                vec![0xBB; 32],
                128,
                3,
                CompressionFormat::None,
                &second_source_path,
            )
            .await
            .unwrap();

        let chunks = segment.chunks().await.unwrap();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], first_entry);
        assert_eq!(chunks[1], second_entry);
        assert!(chunks[0].payload_offset > chunks[0].header_offset);
        assert_eq!(chunks[0].compressed_size, 24);
        assert_eq!(chunks[1].compressed_size, 3);
    }

    #[tokio::test]
    async fn read_chunk_returns_bounded_reader() {
        let tempdir = tempdir().unwrap();
        let segment_path = tempdir.path().join("seg-00000000003.seg");
        let source_path = tempdir.path().join("payload-source.chunk");

        let mut segment = SegmentFile::create_with_created_at(&segment_path, 3, 4096, 77)
            .await
            .unwrap();

        create_source_file(&source_path, b"payload-123").await;
        let entry = segment
            .append_chunk_from_path(
                vec![0xCC; 32],
                42,
                11,
                CompressionFormat::Brotli,
                &source_path,
            )
            .await
            .unwrap();

        let (read_entry, mut reader) = segment.read_chunk_at(entry.header_offset).await.unwrap();
        let mut payload = Vec::new();
        reader.read_to_end(&mut payload).await.unwrap();

        assert_eq!(read_entry, entry);
        assert_eq!(payload, b"payload-123");

        let mut second_reader = segment.chunk_reader(&entry).await.unwrap();
        let mut second_payload = Vec::new();
        second_reader
            .read_to_end(&mut second_payload)
            .await
            .unwrap();
        assert_eq!(second_payload, b"payload-123");
    }

    #[tokio::test]
    async fn append_chunk_from_path_rejects_wrong_size() {
        let tempdir = tempdir().unwrap();
        let segment_path = tempdir.path().join("seg-00000000004.seg");
        let source_path = tempdir.path().join("transient.chunk");

        let mut segment = SegmentFile::create_with_created_at(&segment_path, 4, 4096, 99)
            .await
            .unwrap();

        create_source_file(&source_path, b"transient").await;

        let error = segment
            .append_chunk_from_path(
                vec![0xDD; 32],
                10,
                10,
                CompressionFormat::Zlib,
                &source_path,
            )
            .await
            .unwrap_err();

        let chunks = segment.chunks().await.unwrap();

        assert!(error.to_string().contains("segment source size mismatch"));
        assert!(chunks.is_empty());
        assert_eq!(segment.size_total(), segment.data_offset());
    }

    #[tokio::test]
    async fn append_chunk_from_reader_reuses_existing_payload_stream() {
        let tempdir = tempdir().unwrap();
        let segment_path = tempdir.path().join("seg-00000000004b.seg");

        let mut segment = SegmentFile::create_with_created_at(&segment_path, 41, 4096, 99)
            .await
            .unwrap();
        let mut reader = std::io::Cursor::new(b"reader-payload".to_vec());

        let entry = segment
            .append_chunk_from_reader(vec![0xDE; 32], 18, 14, CompressionFormat::Zstd, &mut reader)
            .await
            .unwrap();

        let (read_entry, mut payload_reader) =
            segment.read_chunk_at(entry.header_offset).await.unwrap();
        let mut payload = Vec::new();
        payload_reader.read_to_end(&mut payload).await.unwrap();

        assert_eq!(read_entry, entry);
        assert_eq!(payload, b"reader-payload");
    }

    #[tokio::test]
    async fn segment_reports_full_after_large_append() {
        let tempdir = tempdir().unwrap();
        let segment_path = tempdir.path().join("seg-00000000005.seg");
        let source_path = tempdir.path().join("large-source.chunk");

        let mut segment = SegmentFile::create_with_created_at(&segment_path, 5, 32, 100)
            .await
            .unwrap();

        create_source_file(&source_path, b"0123456789abcdef0123456789abcdef").await;
        segment
            .append_chunk_from_path(
                vec![0xEE; 32],
                20,
                32,
                CompressionFormat::None,
                &source_path,
            )
            .await
            .unwrap();

        assert!(segment.is_full());
        assert_eq!(segment.state(), SegmentFileState::Full);
        assert_eq!(segment.remaining_capacity(), 0);
    }

    #[tokio::test]
    async fn listing_chunks_ignores_truncated_tail() {
        let tempdir = tempdir().unwrap();
        let segment_path = tempdir.path().join("seg-00000000006.seg");
        let source_path = tempdir.path().join("stable-source.chunk");

        let mut segment = SegmentFile::create_with_created_at(&segment_path, 6, 4096, 101)
            .await
            .unwrap();

        create_source_file(&source_path, b"stable").await;
        let entry = segment
            .append_chunk_from_path(vec![0xAB; 32], 7, 6, CompressionFormat::None, &source_path)
            .await
            .unwrap();

        let mut file = OpenOptions::new()
            .append(true)
            .open(&segment_path)
            .await
            .unwrap();
        let partial_header = PoolV3ChunkHeader {
            hash: vec![0xFE; 32],
            size: 9,
            compressed_size: 32,
            compression_format: CompressionFormat::None.as_u32(),
        }
        .encode_length_delimited_to_vec();
        file.write_all(&partial_header).await.unwrap();
        file.write_all(b"short").await.unwrap();
        file.shutdown().await.unwrap();

        let chunks = segment.chunks().await.unwrap();

        assert_eq!(chunks, vec![entry]);
    }

    #[tokio::test]
    async fn scan_chunk_count_recovers_without_auxiliary_metadata() {
        let tempdir = tempdir().unwrap();
        let segment_path = tempdir.path().join("seg-00000000007.seg");
        let source_path = tempdir.path().join("payload.chunk");

        let mut segment = SegmentFile::create_with_created_at(&segment_path, 7, 4096, 202)
            .await
            .unwrap();

        create_source_file(&source_path, b"payload").await;
        segment
            .append_chunk_from_path(vec![0xBC; 32], 9, 7, CompressionFormat::None, &source_path)
            .await
            .unwrap();

        let scanned_chunk_count = SegmentFile::scan_chunk_count(&segment_path).await.unwrap();

        assert_eq!(scanned_chunk_count, 1);
    }
}

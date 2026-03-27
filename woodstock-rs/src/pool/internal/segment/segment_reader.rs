use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use eyre::{bail, eyre, Result};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, BufReader, SeekFrom};

use super::segment_protobuf::SegmentChunkHeader;
use crate::proto::read_length_delimited_message;
use crate::utils::compression::CompressionFormat;

use super::segment_metadata::open_segment_metadata;
use super::segment_model::{SegmentChunkEntry, SegmentFileHeader, SegmentFileState};

/// Reader limited to the exact compressed payload length of one chunk entry.
pub type SegmentChunkReader = tokio::io::Take<File>;

/// Persistent read-only access to one append-only segment file.
#[derive(Debug)]
pub struct SegmentReader {
    path: PathBuf,
    header: SegmentFileHeader,
    size_total: u64,
    data_offset: u64,
    file: File,
}

impl SegmentReader {
    /// Opens an existing segment file for repeated read operations.
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let metadata = open_segment_metadata(path).await?;
        let file = File::open(&metadata.path).await?;

        Ok(Self {
            path: metadata.path,
            header: metadata.header,
            size_total: metadata.size_total,
            data_offset: metadata.data_offset,
            file,
        })
    }

    /// Counts complete chunk entries by scanning the segment file directly.
    pub async fn scan_chunk_count<P: AsRef<Path>>(path: P) -> Result<u64> {
        let segment = Self::open(path).await?;
        u64::try_from(segment.chunks().await?.len()).map_err(Into::into)
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

    /// Returns the total physical size observed when the reader was opened.
    #[must_use]
    pub fn size_total(&self) -> u64 {
        self.size_total
    }

    /// Returns the byte offset where chunk entries start.
    #[must_use]
    pub fn data_offset(&self) -> u64 {
        self.data_offset
    }

    /// Returns whether the segment had reached its target size when opened.
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.size_total >= self.header.target_size
    }

    /// Returns the derived state snapshot observed when the reader was opened.
    #[must_use]
    pub fn state(&self) -> SegmentFileState {
        if self.is_full() {
            SegmentFileState::Full
        } else {
            SegmentFileState::Open
        }
    }

    /// Returns how many more bytes could still be appended based on the open-time snapshot.
    #[must_use]
    pub fn remaining_capacity(&self) -> u64 {
        self.header.target_size.saturating_sub(self.size_total)
    }

    /// Scans the segment and returns every complete chunk entry currently visible on disk.
    pub async fn chunks(&self) -> Result<Vec<SegmentChunkEntry>> {
        let file = self.file.try_clone().await?;
        let file_size = file.metadata().await?.len();
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(self.data_offset)).await?;

        let mut chunks = Vec::new();

        loop {
            let header_offset = reader.stream_position().await?;

            let (chunk_header, chunk_header_size) =
                match read_length_delimited_message::<_, SegmentChunkHeader>(&mut reader).await? {
                    Some(result) => result,
                    None => break,
                };
            let chunk_header_size = u64::try_from(chunk_header_size)?;

            let payload_offset = reader.stream_position().await?;
            let next_chunk_offset = match payload_offset.checked_add(chunk_header.compressed_size) {
                Some(offset) => offset,
                None => bail!("segment chunk payload offset overflow"),
            };

            if next_chunk_offset > file_size {
                break;
            }

            reader.seek(SeekFrom::Start(next_chunk_offset)).await?;

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
    pub async fn read_chunk_at(
        &self,
        header_offset: u64,
    ) -> Result<(SegmentChunkEntry, SegmentChunkReader)> {
        let mut file = self.file.try_clone().await?;
        file.seek(SeekFrom::Start(header_offset)).await?;
        let Some((chunk_header, chunk_header_size)) =
            read_length_delimited_message::<_, SegmentChunkHeader>(&mut file).await?
        else {
            bail!("segment chunk header is truncated or missing at offset {header_offset}");
        };
        let chunk_header_size = u64::try_from(chunk_header_size)?;
        let payload_offset = file.stream_position().await?;
        let required_len = payload_offset
            .checked_add(chunk_header.compressed_size)
            .ok_or_else(|| eyre!("segment chunk payload offset overflow"))?;

        if file.metadata().await?.len() < required_len {
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
        let compressed_size = entry.compressed_size;
        Ok((entry, file.take(compressed_size)))
    }

    /// Opens a bounded reader for an already known chunk entry.
    pub async fn chunk_reader(&self, entry: &SegmentChunkEntry) -> Result<SegmentChunkReader> {
        let mut file = self.file.try_clone().await?;
        file.seek(SeekFrom::Start(entry.payload_offset)).await?;
        Ok(file.take(entry.compressed_size))
    }
}

#[cfg(test)]
mod tests {
    use prost::Message;
    use tempfile::tempdir;
    use tokio::fs::{File, OpenOptions};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::super::segment_writer::SegmentWriter;
    use super::*;

    async fn create_source_file(path: &Path, payload: &[u8]) {
        let mut file = File::create(path).await.unwrap();
        file.write_all(payload).await.unwrap();
        file.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn append_and_list_chunks() {
        let tempdir = tempdir().unwrap();
        let segment_path = tempdir.path().join("seg-00000000002.seg");
        let first_source_path = tempdir.path().join("first-source.chunk");
        let second_source_path = tempdir.path().join("second-source.chunk");

        let mut segment = SegmentWriter::create_with_created_at(&segment_path, 2, 4096, 55)
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

        let reader = SegmentReader::open(&segment_path).await.unwrap();
        let chunks = reader.chunks().await.unwrap();
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

        let mut writer = SegmentWriter::create_with_created_at(&segment_path, 3, 4096, 77)
            .await
            .unwrap();

        create_source_file(&source_path, b"payload-123").await;
        let entry = writer
            .append_chunk_from_path(
                vec![0xCC; 32],
                42,
                11,
                CompressionFormat::Brotli,
                &source_path,
            )
            .await
            .unwrap();

        let reader = SegmentReader::open(&segment_path).await.unwrap();
        let (read_entry, mut chunk_reader) =
            reader.read_chunk_at(entry.header_offset).await.unwrap();
        let mut payload = Vec::new();
        chunk_reader.read_to_end(&mut payload).await.unwrap();

        assert_eq!(read_entry, entry);
        assert_eq!(payload, b"payload-123");

        let mut second_reader = reader.chunk_reader(&entry).await.unwrap();
        let mut second_payload = Vec::new();
        second_reader
            .read_to_end(&mut second_payload)
            .await
            .unwrap();
        assert_eq!(second_payload, b"payload-123");
    }

    #[tokio::test]
    async fn listing_chunks_ignores_truncated_tail() {
        let tempdir = tempdir().unwrap();
        let segment_path = tempdir.path().join("seg-00000000006.seg");
        let source_path = tempdir.path().join("stable-source.chunk");

        let mut segment = SegmentWriter::create_with_created_at(&segment_path, 6, 4096, 101)
            .await
            .unwrap();

        create_source_file(&source_path, b"stable").await;
        let entry = segment
            .append_chunk_from_path(vec![0xAB; 32], 7, 6, CompressionFormat::None, &source_path)
            .await
            .unwrap();
        segment.shutdown().await.unwrap();

        let mut file = OpenOptions::new()
            .append(true)
            .open(&segment_path)
            .await
            .unwrap();
        let partial_header = SegmentChunkHeader {
            hash: vec![0xFE; 32],
            size: 9,
            compressed_size: 32,
            compression_format: CompressionFormat::None.as_u32(),
        }
        .encode_length_delimited_to_vec();
        file.write_all(&partial_header).await.unwrap();
        file.write_all(b"short").await.unwrap();
        file.shutdown().await.unwrap();

        let reader = SegmentReader::open(&segment_path).await.unwrap();
        let chunks = reader.chunks().await.unwrap();

        assert_eq!(chunks, vec![entry]);
    }

    #[tokio::test]
    async fn scan_chunk_count_recovers_without_auxiliary_metadata() {
        let tempdir = tempdir().unwrap();
        let segment_path = tempdir.path().join("seg-00000000007.seg");
        let source_path = tempdir.path().join("payload.chunk");

        let mut segment = SegmentWriter::create_with_created_at(&segment_path, 7, 4096, 202)
            .await
            .unwrap();

        create_source_file(&source_path, b"payload").await;
        segment
            .append_chunk_from_path(vec![0xBC; 32], 9, 7, CompressionFormat::None, &source_path)
            .await
            .unwrap();
        segment.shutdown().await.unwrap();

        let scanned_chunk_count = SegmentReader::scan_chunk_count(&segment_path)
            .await
            .unwrap();

        assert_eq!(scanned_chunk_count, 1);
    }
}

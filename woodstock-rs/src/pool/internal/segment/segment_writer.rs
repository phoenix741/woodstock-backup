use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use eyre::{bail, eyre, Result, WrapErr};
use tokio::fs::{create_dir_all, metadata, File, OpenOptions};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};

use super::segment_metadata::read_persisted_segment_file_metadata;
use super::segment_protobuf::{SegmentChunkHeader, SegmentHeader};
use crate::proto::write_length_delimited_message;
use crate::utils::compression::CompressionFormat;

use super::segment_metadata::{open_segment_metadata, write_segment_file_metadata};
use super::segment_model::{
    SegmentChunkEntry, SegmentFileHeader, SegmentFileMetadata, SegmentFileState,
    SEGMENT_FORMAT_VERSION,
};

/// Persistent append-only writer for one segment file.
#[derive(Debug)]
pub struct SegmentWriter {
    path: PathBuf,
    header: SegmentFileHeader,
    file_metadata: SegmentFileMetadata,
    size_total: u64,
    data_offset: u64,
    file: File,
    shutdown: bool,
}

impl SegmentWriter {
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

        let header_record = SegmentHeader {
            format_version: SEGMENT_FORMAT_VERSION,
            segment_id,
            target_size,
            created_at,
        };
        let mut file = File::create(&path).await?;
        let header_size =
            u64::try_from(write_length_delimited_message(&mut file, &header_record).await?)?;
        file.flush().await?;

        let file_metadata = SegmentFileMetadata {
            segment_id,
            state: SegmentFileState::Open,
            size_total: header_size,
            size_effective: 0,
            size_limit: target_size,
            chunk_count: 0,
            dead_stored_bytes: 0,
        };
        write_segment_file_metadata(&path, &file_metadata).await?;

        Ok(Self {
            path,
            header: header_record.into(),
            file_metadata,
            size_total: header_size,
            data_offset: header_size,
            file,
            shutdown: false,
        })
    }

    /// Opens an existing segment file and keeps an append handle alive.
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let metadata = open_segment_metadata(path).await?;
        let file_metadata = read_persisted_segment_file_metadata(&metadata.path)
            .await
            .wrap_err("segment metadata sidecar is missing")?;
        let file = OpenOptions::new().append(true).open(&metadata.path).await?;

        // Guard against crash-recovery skew: if the sidecar recorded a size_total
        // larger than the actual file, the previous write was not fully flushed.
        // Writing at that offset would create a sparse hole and corrupt earlier chunks.
        let actual_len = file.metadata().await?.len();
        if file_metadata.size_total > actual_len {
            bail!(
                "segment file is truncated: sidecar reports {} bytes but file is {} bytes ({})",
                file_metadata.size_total,
                actual_len,
                metadata.path.display()
            );
        }

        Ok(Self {
            path: metadata.path,
            header: metadata.header,
            file_metadata,
            size_total: metadata.size_total,
            data_offset: metadata.data_offset,
            file,
            shutdown: false,
        })
    }

    /// Flushes and closes the persistent writer handle.
    pub async fn shutdown(&mut self) -> Result<()> {
        self.shutdown = true;
        write_segment_file_metadata(&self.path, &self.file_metadata).await?;

        self.file.flush().await?;
        self.file.shutdown().await?;
        Ok(())
    }

    /// Returns the filesystem path of the segment file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the immutable segment header.
    #[must_use]
    pub fn header(&self) -> &SegmentFileHeader {
        &self.header
    }

    /// Returns the persisted sidecar metadata tracked for this segment file.
    #[must_use]
    pub fn file_metadata(&self) -> &SegmentFileMetadata {
        &self.file_metadata
    }

    /// Returns the current total physical size tracked by the writer.
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

    /// Appends a chunk payload already stored in a source file.
    ///
    /// Opens the file at `source_path`, validates its length against
    /// `compressed_size`, and copies the payload with a buffered async copy.
    ///
    /// # Errors
    ///
    /// Returns an error if the writer is shut down, the file size does not match
    /// `compressed_size`, or any I/O operation fails.
    pub async fn append_chunk_from_path(
        &mut self,
        hash: Vec<u8>,
        size: u64,
        compressed_size: u64,
        compression_format: CompressionFormat,
        source_path: &Path,
    ) -> Result<SegmentChunkEntry> {
        if self.shutdown {
            bail!("segment writer is shut down");
        }
        let source_size = metadata(source_path).await?.len();
        if source_size != compressed_size {
            bail!(
                "segment source size mismatch: expected {compressed_size} bytes, found {source_size} bytes"
            );
        }

        let source_file = File::open(source_path).await?;
        self.append_chunk_from_reader(hash, size, compressed_size, compression_format, source_file)
            .await
    }

    /// Appends a chunk payload from an arbitrary [`AsyncRead`] source.
    ///
    /// Exactly `compressed_size` bytes are read from `reader` and appended to
    /// the segment.  The caller is responsible for ensuring that `reader` yields
    /// exactly that many bytes (e.g. by wrapping it in [`tokio::io::Take`]).
    ///
    /// This is the preferred path when the payload is already open (e.g.
    /// during segment compaction, where the source is a [`SegmentChunkReader`]
    /// obtained from [`super::segment_reader::SegmentReader::chunk_reader`]),
    /// as no temporary file or path lookup is needed.
    ///
    /// # Errors
    ///
    /// Returns an error if the writer is shut down or any I/O operation fails.
    pub async fn append_chunk_from_reader(
        &mut self,
        hash: Vec<u8>,
        size: u64,
        compressed_size: u64,
        compression_format: CompressionFormat,
        reader: impl AsyncRead + Unpin,
    ) -> Result<SegmentChunkEntry> {
        if self.shutdown {
            bail!("segment writer is shut down");
        }

        let chunk_header = SegmentChunkHeader {
            hash,
            size,
            compressed_size,
            compression_format: compression_format.as_u32(),
        };
        let header_offset = self.size_total;

        // Write the chunk header and record the exact serialised byte count.
        let written_header_size = {
            let written = write_length_delimited_message(&mut self.file, &chunk_header).await?;
            u64::try_from(written)?
        };

        let payload_offset = header_offset
            .checked_add(written_header_size)
            .ok_or_else(|| eyre!("segment chunk payload offset overflow"))?;

        // Copy exactly `compressed_size` bytes from the reader into the file.
        let copied = tokio::io::copy(&mut reader.take(compressed_size), &mut self.file).await?;
        if copied != compressed_size {
            bail!("short read during chunk append: expected {compressed_size} bytes, got {copied}");
        }

        self.size_total = self
            .size_total
            .checked_add(written_header_size)
            .and_then(|v| v.checked_add(compressed_size))
            .ok_or_else(|| eyre!("segment size overflow"))?;
        self.file_metadata.size_total = self.size_total;
        self.file_metadata.size_effective = self
            .file_metadata
            .size_effective
            .checked_add(size)
            .ok_or_else(|| eyre!("segment effective size overflow"))?;
        self.file_metadata.chunk_count = self
            .file_metadata
            .chunk_count
            .checked_add(1)
            .ok_or_else(|| eyre!("segment chunk count overflow"))?;
        self.file_metadata.state = self.state();

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
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use tokio::fs::{metadata, remove_file, File};
    use tokio::io::AsyncWriteExt;

    use super::super::segment_metadata::{
        read_persisted_segment_file_metadata, segment_sidecar_metadata_path,
        write_segment_file_metadata,
    };
    use super::super::segment_reader::SegmentReader;
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

        let mut segment = SegmentWriter::create_with_created_at(&segment_path, 1, 1024, 1234)
            .await
            .unwrap();

        assert_eq!(segment.header().segment_id, 1);
        assert_eq!(segment.header().target_size, 1024);
        assert_eq!(segment.header().created_at, 1234);
        assert_eq!(segment.state(), SegmentFileState::Open);
        assert_eq!(segment.size_total(), segment.data_offset());

        segment.shutdown().await.unwrap();

        let reopened = SegmentReader::open(&segment_path).await.unwrap();

        assert_eq!(reopened.header(), segment.header());
        assert_eq!(reopened.size_total(), segment.size_total());
        assert_eq!(reopened.data_offset(), segment.data_offset());
    }

    #[tokio::test]
    async fn append_chunk_from_path_rejects_wrong_size() {
        let tempdir = tempdir().unwrap();
        let segment_path = tempdir.path().join("seg-00000000004.seg");
        let source_path = tempdir.path().join("transient.chunk");

        let mut segment = SegmentWriter::create_with_created_at(&segment_path, 4, 4096, 99)
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

        let reader = SegmentReader::open(&segment_path).await.unwrap();
        let chunks = reader.chunks().await.unwrap();

        assert!(error.to_string().contains("segment source size mismatch"));
        assert!(chunks.is_empty());
        assert_eq!(segment.size_total(), segment.data_offset());
    }

    #[tokio::test]
    async fn segment_reports_full_after_large_append() {
        let tempdir = tempdir().unwrap();
        let segment_path = tempdir.path().join("seg-00000000005.seg");
        let source_path = tempdir.path().join("large-source.chunk");

        let mut segment = SegmentWriter::create_with_created_at(&segment_path, 5, 32, 100)
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
    async fn reader_can_observe_flushed_chunk_before_writer_shutdown() {
        let tempdir = tempdir().unwrap();
        let segment_path = tempdir.path().join("seg-00000000008.seg");
        let source_path = tempdir.path().join("payload-visible.chunk");

        let mut writer = SegmentWriter::create_with_created_at(&segment_path, 8, 4096, 303)
            .await
            .unwrap();

        create_source_file(&source_path, b"visible").await;
        let entry = writer
            .append_chunk_from_path(vec![0xCD; 32], 10, 7, CompressionFormat::None, &source_path)
            .await
            .unwrap();

        let reader = SegmentReader::open(&segment_path).await.unwrap();
        let chunks = reader.chunks().await.unwrap();
        let (read_entry, mut chunk_reader) =
            reader.read_chunk_at(entry.header_offset).await.unwrap();
        let mut payload = Vec::new();
        tokio::io::AsyncReadExt::read_to_end(&mut chunk_reader, &mut payload)
            .await
            .unwrap();

        assert_eq!(chunks, vec![entry.clone()]);
        assert_eq!(read_entry, entry);
        assert_eq!(payload, b"visible");
    }

    #[tokio::test]
    async fn append_after_shutdown_is_rejected() {
        let tempdir = tempdir().unwrap();
        let segment_path = tempdir.path().join("seg-00000000009.seg");
        let source_path = tempdir.path().join("closed.chunk");

        let mut writer = SegmentWriter::create_with_created_at(&segment_path, 9, 4096, 404)
            .await
            .unwrap();
        create_source_file(&source_path, b"closed").await;
        writer.shutdown().await.unwrap();

        let error = writer
            .append_chunk_from_path(vec![0xEF; 32], 10, 6, CompressionFormat::None, &source_path)
            .await
            .unwrap_err();

        assert!(error.to_string().contains("segment writer is shut down"));
    }

    #[tokio::test]
    async fn open_reads_existing_segment_metadata() {
        let tempdir = tempdir().unwrap();
        let segment_path = tempdir.path().join("seg-00000000010.seg");

        let mut created = SegmentWriter::create_with_created_at(&segment_path, 10, 2048, 505)
            .await
            .unwrap();
        created.shutdown().await.unwrap();

        let reopened = SegmentWriter::open(&segment_path).await.unwrap();

        assert_eq!(reopened.path(), segment_path.as_path());
        assert_eq!(reopened.header().segment_id, 10);
        assert_eq!(reopened.header().target_size, 2048);
        assert_eq!(reopened.header().created_at, 505);
        assert_eq!(reopened.size_total(), reopened.data_offset());
    }

    #[tokio::test]
    async fn create_writes_sidecar_metadata_file() {
        let tempdir = tempdir().unwrap();
        let segment_path = tempdir.path().join("seg-00000000011.seg");

        let writer = SegmentWriter::create_with_created_at(&segment_path, 11, 4096, 606)
            .await
            .unwrap();
        let sidecar_path = segment_sidecar_metadata_path(&segment_path);
        let file_metadata = read_persisted_segment_file_metadata(&segment_path)
            .await
            .unwrap();

        assert_eq!(
            metadata(&sidecar_path).await.unwrap().len(),
            sidecar_path.metadata().unwrap().len()
        );
        assert_eq!(file_metadata, *writer.file_metadata());
        assert_eq!(file_metadata.segment_id, 11);
        assert_eq!(file_metadata.state, SegmentFileState::Open);
        assert_eq!(file_metadata.size_total, writer.size_total());
        assert_eq!(file_metadata.size_effective, 0);
        assert_eq!(file_metadata.size_limit, 4096);
        assert_eq!(file_metadata.chunk_count, 0);
    }

    #[tokio::test]
    async fn append_updates_sidecar_metadata_file() {
        let tempdir = tempdir().unwrap();
        let segment_path = tempdir.path().join("seg-00000000012.seg");
        let source_path = tempdir.path().join("meta-source.chunk");

        let mut writer = SegmentWriter::create_with_created_at(&segment_path, 12, 4096, 707)
            .await
            .unwrap();
        create_source_file(&source_path, b"metadata").await;

        writer
            .append_chunk_from_path(vec![0xAA; 32], 32, 8, CompressionFormat::None, &source_path)
            .await
            .unwrap();
        writer.shutdown().await.unwrap();

        let file_metadata = read_persisted_segment_file_metadata(&segment_path)
            .await
            .unwrap();

        assert_eq!(file_metadata, *writer.file_metadata());
        assert_eq!(file_metadata.size_total, writer.size_total());
        assert_eq!(file_metadata.size_effective, 32);
        assert_eq!(file_metadata.chunk_count, 1);
    }

    #[tokio::test]
    async fn open_rejects_missing_sidecar_metadata_file() {
        let tempdir = tempdir().unwrap();
        let segment_path = tempdir.path().join("seg-00000000013.seg");
        let source_path = tempdir.path().join("missing-sidecar-source.chunk");

        let mut writer = SegmentWriter::create_with_created_at(&segment_path, 13, 4096, 808)
            .await
            .unwrap();
        create_source_file(&source_path, b"missing").await;
        writer
            .append_chunk_from_path(vec![0xBB; 32], 64, 7, CompressionFormat::None, &source_path)
            .await
            .unwrap();
        writer.shutdown().await.unwrap();

        let sidecar_path = segment_sidecar_metadata_path(&segment_path);
        remove_file(&sidecar_path).await.unwrap();
        assert!(read_persisted_segment_file_metadata(&segment_path)
            .await
            .is_err());

        let error = SegmentWriter::open(&segment_path).await.unwrap_err();

        assert!(error
            .to_string()
            .contains("segment metadata sidecar is missing"));
    }

    #[tokio::test]
    async fn open_rejects_sidecar_size_larger_than_file() {
        let tempdir = tempdir().unwrap();
        let segment_path = tempdir.path().join("seg-00000000014.seg");
        let source_path = tempdir.path().join("partial-source.chunk");

        let mut writer = SegmentWriter::create_with_created_at(&segment_path, 14, 4096, 909)
            .await
            .unwrap();
        create_source_file(&source_path, b"partial").await;
        writer
            .append_chunk_from_path(vec![0xCC; 32], 16, 7, CompressionFormat::None, &source_path)
            .await
            .unwrap();
        writer.shutdown().await.unwrap();

        // Simulate crash-recovery skew: sidecar claims more bytes than the file holds.
        let mut inflated = read_persisted_segment_file_metadata(&segment_path)
            .await
            .unwrap();
        inflated.size_total += 1024;
        write_segment_file_metadata(&segment_path, &inflated)
            .await
            .unwrap();

        let error = SegmentWriter::open(&segment_path).await.unwrap_err();

        assert!(error.to_string().contains("segment file is truncated"));
    }
}

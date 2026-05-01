use std::path::{Path, PathBuf};
use std::sync::Arc;

use eyre::{eyre, Result};
use tokio::fs::{create_dir_all, metadata, remove_file, File};
use tokio::io::AsyncWriteExt;
use tracing::warn;
use uuid::Uuid;

use crate::config::Configuration;
use crate::utils::chunk_hasher::{create_chunk_hasher, ChunkHasher};
use crate::utils::compression::WoodstockCompressionWriter;

use super::{
    ChunkDescriptor, ChunkIndex, PoolChunkInformation, Segments, SegmentsWriter, StagingWriter,
    STAGING_FILENAME,
};

/// Backup write session that centralizes chunk writes to pool v3.
///
/// The session keeps a long-lived [`SegmentsWriter`] and a per-backup
/// [`StagingWriter`] for the whole backup execution.
pub struct PoolWriteSession {
    config: Arc<Configuration>,
    backup_dir: PathBuf,
    index: ChunkIndex,
    writer: SegmentsWriter,
    staging: StagingWriter,
    staging_compacted: bool,
}

/// Streaming writer for one in-flight chunk payload.
pub struct PoolChunkWriteHandle {
    temp_path: PathBuf,
    writer: WoodstockCompressionWriter<File>,
    hasher: Box<dyn ChunkHasher + Send + Sync>,
    uncompressed_size: u64,
}

struct FinalizedChunkTemp {
    hash: Vec<u8>,
    uncompressed_size: u64,
    compressed_size: u64,
    temp_path: PathBuf,
}

impl PoolWriteSession {
    /// Opens or creates a write session for one backup.
    pub async fn open_or_create(config: Arc<Configuration>, backup_dir: &Path) -> Result<Self> {
        let backup_dir = backup_dir.to_path_buf();
        create_dir_all(&backup_dir).await?;

        let segments = Segments::new(Arc::clone(&config));
        let writer = segments.get_writer().await?;
        let staging_path = backup_dir.join(STAGING_FILENAME);
        let staging = if staging_path.exists() {
            StagingWriter::open(&backup_dir).await?
        } else {
            StagingWriter::create(&backup_dir).await?
        };

        Ok(Self {
            index: ChunkIndex::new(Arc::clone(&config)),
            config,
            backup_dir,
            writer,
            staging,
            staging_compacted: false,
        })
    }

    /// Returns chunk information if the hash is already known in staging or index.
    pub async fn find_chunk(&mut self, hash: &[u8]) -> Result<Option<PoolChunkInformation>> {
        if let Some(desc) = self.staging.get_descriptor(hash) {
            return Ok(Some(descriptor_to_chunk_information(&desc)));
        }

        let from_index = self.index.get_chunk(hash).await?;
        Ok(from_index.as_ref().map(descriptor_to_chunk_information))
    }

    /// Creates a streamed chunk writer backed by a temporary compressed file.
    pub async fn create_chunk(&self) -> Result<PoolChunkWriteHandle> {
        let tmp_path = self
            .config
            .path
            .pool_path
            .join("tmp")
            .join(format!("segment-upload-{}.tmp", Uuid::new_v4()));

        if let Some(parent) = tmp_path.parent() {
            create_dir_all(parent).await?;
        }

        let file = File::create(&tmp_path).await?;
        let writer = WoodstockCompressionWriter::new(
            tokio::io::BufWriter::new(file),
            self.config.compression_format,
        );

        Ok(PoolChunkWriteHandle {
            temp_path: tmp_path,
            writer,
            hasher: create_chunk_hasher(self.config.chunk_algorithm),
            uncompressed_size: 0,
        })
    }

    async fn integrate_finished_chunk(
        &mut self,
        finalized: FinalizedChunkTemp,
    ) -> Result<PoolChunkInformation> {
        if let Some(existing) = self.lookup_descriptor(&finalized.hash).await? {
            if let Err(e) = remove_file(&finalized.temp_path).await {
                warn!(
                    "Failed to remove temporary compressed chunk {:?}: {}",
                    finalized.temp_path, e
                );
            }
            let mut staging_desc = existing.clone();
            staging_desc.refcount = 1;
            self.staging.write(&staging_desc).await?;
            return Ok(descriptor_to_chunk_information(&existing));
        }

        let segment_id = self.writer.header().segment_id;
        let entry = self
            .writer
            .append_chunk_from_path(
                finalized.hash.clone(),
                finalized.uncompressed_size,
                finalized.compressed_size,
                self.config.compression_format,
                &finalized.temp_path,
            )
            .await?;

        if let Err(e) = remove_file(&finalized.temp_path).await {
            warn!(
                "Failed to remove temporary compressed chunk {:?}: {}",
                finalized.temp_path, e
            );
        }

        let header_size = u32::try_from(entry.chunk_header_size)
            .map_err(|_| eyre!("chunk_header_size does not fit in u32"))?;

        let desc = ChunkDescriptor {
            hash: entry.hash.clone(),
            segment_id,
            offset: entry.header_offset,
            size: entry.size,
            compressed_size: entry.compressed_size,
            header_size,
            compression_format: entry.compression_format.as_u32(),
            refcount: 1,
        };

        self.staging.write(&desc).await?;

        Ok(descriptor_to_chunk_information(&desc))
    }

    /// Compacts and flushes staging once for this session.
    pub async fn compact_staging(&mut self) -> Result<()> {
        if self.staging_compacted {
            return Ok(());
        }
        self.staging.compact().await?;
        self.staging_compacted = true;
        Ok(())
    }

    /// Flushes writers and releases current segment lock.
    pub async fn shutdown(&mut self) -> Result<()> {
        self.writer.shutdown().await?;
        self.staging.shutdown().await?;
        Ok(())
    }

    /// Compacts staging file in place for a backup directory.
    pub async fn compact_backup_staging(backup_dir: &Path) -> Result<()> {
        let staging_path = backup_dir.join(STAGING_FILENAME);
        if !staging_path.exists() {
            return Ok(());
        }

        let mut writer = StagingWriter::open(backup_dir).await?;
        writer.compact().await?;
        writer.close().await?;
        Ok(())
    }

    /// Integrates backup staging into the global index.
    pub async fn publish_backup_staging(
        config: Arc<Configuration>,
        backup_dir: &Path,
    ) -> Result<()> {
        let staging_path = backup_dir.join(STAGING_FILENAME);
        if !staging_path.exists() {
            return Ok(());
        }

        let mut index = ChunkIndex::new(config);
        let mut writer = index.get_writer().await?;
        writer.add_staging(&staging_path).await?;
        writer.shutdown().await?;
        Ok(())
    }

    async fn lookup_descriptor(&mut self, hash: &[u8]) -> Result<Option<ChunkDescriptor>> {
        if let Some(desc) = self.staging.get_descriptor(hash) {
            return Ok(Some(desc));
        }

        Ok(self.index.get_chunk(hash).await?)
    }

    #[must_use]
    pub fn backup_dir(&self) -> &Path {
        &self.backup_dir
    }
}

impl PoolChunkWriteHandle {
    /// Appends one uncompressed payload fragment to the temporary chunk file.
    pub async fn write(&mut self, data: &[u8]) -> Result<()> {
        self.writer.write_all(data).await?;
        self.hasher.update(data);
        self.uncompressed_size = self
            .uncompressed_size
            .checked_add(u64::try_from(data.len())?)
            .ok_or_else(|| eyre!("chunk uncompressed size overflow"))?;
        Ok(())
    }

    async fn finalize(mut self) -> Result<FinalizedChunkTemp> {
        self.writer.shutdown().await?;

        let compressed_size = metadata(&self.temp_path).await?.len();
        let hash = self.hasher.finalize();

        Ok(FinalizedChunkTemp {
            hash,
            uncompressed_size: self.uncompressed_size,
            compressed_size,
            temp_path: self.temp_path,
        })
    }

    /// Finalizes this chunk writer and integrates the chunk into segment/staging.
    pub async fn finish(self, session: &mut PoolWriteSession) -> Result<PoolChunkInformation> {
        let finalized = self.finalize().await?;
        session.integrate_finished_chunk(finalized).await
    }
}

fn descriptor_to_chunk_information(desc: &ChunkDescriptor) -> PoolChunkInformation {
    PoolChunkInformation {
        sha256: desc.hash.clone(),
        size: desc.size,
        compressed_size: desc.compressed_size,
        format: desc.compression_format,
    }
}

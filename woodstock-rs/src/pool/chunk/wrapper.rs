use eyre::{eyre, Result};
use futures::{pin_mut, Stream, StreamExt};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    fs::{create_dir_all, File},
    io::{self, AsyncRead, AsyncReadExt, AsyncWriteExt},
};
use tracing::{debug, error};

use crate::{
    config::{Configuration, BUFFER_SIZE},
    utils::compression::WoodstockCompressionReader,
    utils::{chunk_hasher::create_chunk_hasher, compression::CompressionFormat},
    ChunkAlgorithm,
};

use super::{PoolChunkInformation, PoolChunkWriter};
use crate::pool::{data, IndexedChunk, PoolManager};

type PoolChunkReader = Box<dyn AsyncRead + Send + Sync + Unpin>;

/// # Pool Chunk Wrapper Module
///
/// This module provides the [`PoolChunkWrapper`] struct and associated methods for managing
/// logical chunks stored in the Woodstock Pool V3. The wrapper uses the materialized protobuf
/// checkpoint and shard index for existence and metadata, and segment files as the physical
/// payload backend.
///
/// ## Main Structure
///
/// - [`PoolChunkWrapper`]: Encapsulates chunk file operations and metadata.
///
/// ## Main Methods
///
/// - [`PoolChunkWrapper::new`]: Create a new wrapper for a chunk.
/// - [`PoolChunkWrapper::exists`]: Check if the chunk file exists.
/// - [`PoolChunkWrapper::writer`]: Create a writer for the chunk.
/// - [`PoolChunkWrapper::chunk_information`]: Read chunk metadata.
/// - [`PoolChunkWrapper::check_chunk_information`]: Verify chunk integrity.
///
/// ## Error Handling & Panics
///
/// - All async methods return `Result` and propagate errors using the `eyre` crate.
/// - Panics are not expected under normal operation, except for assertion on hash presence in `mv`.
///
/// ## See Also
///
/// - [`PoolChunkWriter`], [`PoolChunkInformation`]: For chunk writing and metadata
pub struct PoolChunkWrapper {
    pool_path: PathBuf,
    hash_str: Option<String>,
    hash: Option<Vec<u8>>,
    config: Option<Arc<Configuration>>,
}

impl PoolChunkWrapper {
    #[must_use]
    pub fn new(pool_path: &Path, hash: Option<&Vec<u8>>) -> PoolChunkWrapper {
        let mut wrapper = PoolChunkWrapper {
            pool_path: pool_path.to_path_buf(),
            hash: None,
            hash_str: None,
            config: None,
        };
        wrapper.set_hash(hash);

        wrapper
    }

    #[must_use]
    pub fn with_pool_configuration(mut self, config: Arc<Configuration>) -> Self {
        self.config = Some(config);
        self
    }

    #[must_use]
    pub fn get_hash_str(&self) -> &Option<String> {
        &self.hash_str
    }

    #[must_use]
    pub fn get_hash(&self) -> &Option<Vec<u8>> {
        &self.hash
    }

    pub fn set_hash(&mut self, hash: Option<&Vec<u8>>) {
        self.hash_str = hash.map(hex::encode);
        self.hash = hash.cloned();
    }

    #[must_use]
    pub fn exists(&self) -> bool {
        self.indexed_chunk()
            .map(|chunk| chunk.is_some())
            .unwrap_or(false)
    }

    pub async fn remove(&self) -> std::io::Result<()> {
        let Some(hash) = self.hash.as_deref() else {
            return Ok(());
        };

        let index = data::open_pool_index(&self.pool_path).map_err(io::Error::other)?;
        index.remove_chunk(hash).map_err(io::Error::other)?;
        Ok(())
    }

    pub async fn mv<P: AsRef<Path>>(&self, target_path: P) -> std::io::Result<()> {
        let target_path = target_path.as_ref();
        if let Some(parent) = target_path.parent() {
            create_dir_all(parent).await?;
        }

        let mut reader = self.open_chunk_reader().await.map_err(io::Error::other)?;
        let mut file = File::create(target_path).await?;
        tokio::io::copy(&mut reader, &mut file).await?;
        file.shutdown().await?;

        Ok(())
    }

    pub async fn chunk_information(&self) -> Result<PoolChunkInformation> {
        let chunk = self
            .indexed_chunk()?
            .ok_or_else(|| eyre!("chunk {} is not indexed in Pool V3", self.hash_hex()))?;
        Ok(Self::chunk_information_from_indexed_chunk(chunk))
    }

    pub async fn open_chunk_reader(&self) -> Result<WoodstockCompressionReader<PoolChunkReader>> {
        let chunk = self
            .indexed_chunk()?
            .ok_or_else(|| eyre!("chunk {} is not indexed in Pool V3", self.hash_hex()))?;
        let index = data::open_pool_index(&self.pool_path)?;
        let segment = index.get_segment(chunk.segment_id)?.ok_or_else(|| {
            eyre!(
                "missing pool v3 segment {} for chunk {}",
                chunk.segment_id,
                self.hash_hex()
            )
        })?;
        let segment_file = data::open_indexed_segment(&self.pool_path, &segment).await?;
        let (_, reader) = segment_file.read_chunk_at(chunk.offset).await?;

        Ok(WoodstockCompressionReader::new(Box::new(reader)))
    }

    async fn calculate_chunk_hash(&self, chunk_algorithm: ChunkAlgorithm) -> Result<Vec<u8>> {
        let mut file = self.open_chunk_reader().await?;
        let mut hasher = create_chunk_hasher(chunk_algorithm);

        let mut buffer = vec![0u8; BUFFER_SIZE];
        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        Ok(hasher.finalize())
    }

    pub async fn check_chunk_information(&self, chunk_algorithm: ChunkAlgorithm) -> Result<bool> {
        debug!(
            "Checking chunk information for {} with algorithm {:?}",
            self.hash_hex(),
            chunk_algorithm,
        );
        let file_hash = self.calculate_chunk_hash(chunk_algorithm).await?;

        if let Some(hash) = &self.hash {
            if hash.ne(&file_hash) {
                error!(
                    "When reading the chunk, the hash should be {} but is {}",
                    hex::encode(hash),
                    hex::encode(&file_hash)
                );
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub async fn writer(
        &self,
        chunk_algorithm: ChunkAlgorithm,
        compression_format: CompressionFormat,
    ) -> Result<PoolChunkWriter> {
        let pool_path = self.pool_path.clone();
        PoolChunkWriter::new(&pool_path, chunk_algorithm, compression_format).await
    }

    pub async fn write(
        &mut self,
        data: impl Stream<Item = Result<Vec<u8>>>,
        debug_filename: &[u8],
        chunk_algorithm: ChunkAlgorithm,
        compression_format: CompressionFormat,
    ) -> Result<PoolChunkInformation> {
        let mut writer = self.writer(chunk_algorithm, compression_format).await?;

        pin_mut!(data);

        while let Some(chunk) = data.next().await {
            match chunk {
                Ok(chunk) => {
                    writer.write(&chunk).await?;
                }
                Err(e) => {
                    error!("Error while reading the chunk: {:?}", e);
                    return Err(e);
                }
            };
        }
        let prepared_chunk = writer
            .shutdown(self, debug_filename, compression_format)
            .await?;
        let pool_manager = PoolManager::new(self.pool_path_configuration()?);
        let chunk_information = pool_manager.store_prepared_chunk(prepared_chunk).await?;

        Ok(chunk_information)
    }

    pub async fn copy(
        &self,
        target_chunk: &mut PoolChunkWrapper,
        chunk_algorithm: ChunkAlgorithm,
    ) -> Result<()> {
        let chunk_information = self.chunk_information().await?;
        let compression_format = CompressionFormat::try_from(chunk_information.format)?;
        let mut reader = self.open_chunk_reader().await?;
        let mut writer = target_chunk
            .writer(chunk_algorithm, compression_format)
            .await?;

        let mut buffer = vec![0_u8; BUFFER_SIZE];
        loop {
            let bytes_read = reader.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }
            writer.write(&buffer[..bytes_read]).await?;
        }

        let published_chunk = writer
            .shutdown(target_chunk, b"pool-chunk-copy", compression_format)
            .await?;
        let chunk_information = PoolManager::new(target_chunk.pool_path_configuration()?)
            .store_prepared_chunk(published_chunk)
            .await?;
        let copied_hash = chunk_information.chunk_hash.clone();
        target_chunk.set_hash(Some(&copied_hash));

        Ok(())
    }

    fn indexed_chunk(&self) -> Result<Option<IndexedChunk>> {
        let Some(hash) = self.hash.as_deref() else {
            return Ok(None);
        };

        let index = data::open_pool_index(&self.pool_path)?;
        index.get_chunk(hash)
    }

    fn chunk_information_from_indexed_chunk(chunk: IndexedChunk) -> PoolChunkInformation {
        PoolChunkInformation {
            chunk_hash: chunk.hash,
            size: chunk.size,
            compressed_size: chunk.compressed_size,
            format: chunk.compression_format.as_u32(),
            segment_id: chunk.segment_id,
            offset: chunk.offset,
            chunk_header_size: chunk.chunk_header_size,
        }
    }

    fn hash_hex(&self) -> &str {
        self.hash_str.as_deref().unwrap_or("<unknown>")
    }

    fn pool_path_configuration(&self) -> Result<std::sync::Arc<crate::config::Configuration>> {
        self.config.clone().ok_or_else(|| {
            eyre!(
                "PoolChunkWrapper physical publication requires an explicit Configuration context"
            )
        })
    }
}

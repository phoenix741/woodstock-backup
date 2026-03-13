use eyre::Result;
use futures::{pin_mut, Stream, StreamExt};
use prost::Message;
use std::path::{Path, PathBuf};
use tokio::{
    fs::{copy, create_dir_all, remove_file, rename, File},
    io::{AsyncReadExt, AsyncWriteExt},
};
use tracing::{debug, error, warn};

use crate::utils::{chunk_hasher::create_chunk_hasher, compression::CompressionFormat};
use crate::ChunkAlgorithm;
use crate::{config::BUFFER_SIZE, utils::compression::WoodstockCompressionReader};
use reflink_copy::reflink;

use super::{
    calculate_chunk_path, get_temp_chunk_path, pool_chunk_information::PoolChunkInformation,
    PoolChunkWriter,
};

/// # Pool Chunk Wrapper Module
///
/// This module provides the [`PoolChunkWrapper`] struct and associated methods for managing
/// chunk files in the Woodstock backup pool. It handles chunk path calculation, metadata,
/// reading, writing, moving, and integrity checking.
///
/// ## Main Structure
///
/// - [`PoolChunkWrapper`]: Encapsulates chunk file operations and metadata.
///
/// ## Main Methods
///
/// - [`PoolChunkWrapper::new`]: Create a new wrapper for a chunk.
/// - [`PoolChunkWrapper::chunk_path`]: Get the path to the chunk file.
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
    /// Represents the path to the storage pool.
    ///
    /// This field is used to locate and manage chunk files within the pool.
    pool_path: PathBuf,
    /// Optional string representation of the hash.
    hash_str: Option<String>,
    /// Optional binary representation of the hash.
    hash: Option<Vec<u8>>,
}

impl PoolChunkWrapper {
    /// Creates a new [`PoolChunkWrapper`] for a chunk.
    ///
    /// # Arguments
    ///
    /// * `pool_path` - The root directory of the chunk pool.
    /// * `hash` - Optional hash of the chunk.
    ///
    /// # Returns
    ///
    /// A new instance of [`PoolChunkWrapper`].
    #[must_use]
    pub fn new(pool_path: &Path, hash: Option<&Vec<u8>>) -> PoolChunkWrapper {
        let mut wrapper = PoolChunkWrapper {
            pool_path: pool_path.to_path_buf(),
            hash: None,
            hash_str: None,
        };
        wrapper.set_hash(hash);

        wrapper
    }

    /// Returns the hash of the chunk as a hexadecimal string.
    ///
    /// # Returns
    ///
    /// * `&Option<String>` - The hash string if set, or `None`.
    #[must_use]
    pub fn get_hash_str(&self) -> &Option<String> {
        &self.hash_str
    }

    /// Returns the hash of the chunk as a byte vector.
    ///
    /// # Returns
    ///
    /// * `&Option<Vec<u8>>` - The hash if set, or `None`.
    #[must_use]
    pub fn get_hash(&self) -> &Option<Vec<u8>> {
        &self.hash
    }

    /// Sets the hash of the chunk.
    ///
    /// # Arguments
    ///
    /// * `hash` - Optional hash of the chunk.
    pub fn set_hash(&mut self, hash: Option<&Vec<u8>>) {
        self.hash_str = hash.map(hex::encode);
        self.hash = hash.cloned();
    }

    /// Checks if the chunk file exists in the pool.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if the chunk file exists, `false` otherwise.
    #[must_use]
    pub fn exists(&self) -> bool {
        self.chunk_path().exists()
    }

    /// Removes the chunk file from the pool.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the file was removed successfully.
    /// * `Err(std::io::Error)` if an error occurs.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be removed due to I/O issues.
    pub async fn remove(&self) -> std::io::Result<()> {
        if let Err(err) = remove_file(self.chunk_path()).await {
            if err.kind() != std::io::ErrorKind::NotFound {
                error!("Failed to remove chunk file: {:?}", err);
                return Err(err);
            }
        }
        if let Err(err) = remove_file(self.chunk_path_information()).await {
            if err.kind() != std::io::ErrorKind::NotFound {
                error!("Failed to remove chunk information file: {:?}", err);
                return Err(err);
            }
        }
        Ok(())
    }

    /// Moves the chunk file to a new location.
    ///
    /// # Arguments
    ///
    /// * `target_path` - The target path for the chunk file.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the file was moved successfully.
    /// * `Err(std::io::Error)` if an error occurs.
    ///
    /// # Panics
    ///
    /// Panics if the hash is not set.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be moved due to I/O issues.
    pub async fn mv<P: AsRef<Path>>(&self, target_path: P) -> std::io::Result<()> {
        assert_ne!(self.hash_str, None, "Hash of the file shouldn't be None");

        if rename(self.chunk_path(), &target_path).await.is_err() {
            // copy the file if the rename fails
            copy(self.chunk_path(), &target_path).await?;
            self.remove().await?;
        }

        Ok(())
    }

    /// Returns the path to the chunk file.
    ///
    /// # Returns
    ///
    /// * `PathBuf` - The path to the chunk file.
    #[must_use]
    pub fn chunk_path(&self) -> PathBuf {
        match &self.hash_str {
            Some(hash) => calculate_chunk_path(&self.pool_path, hash),
            None => get_temp_chunk_path(&self.pool_path),
        }
    }

    /// Retrieves the path to the chunk's metadata file.
    fn chunk_path_information(&self) -> PathBuf {
        self.chunk_path().with_extension("info")
    }

    /// Writes metadata for the chunk to a file.
    ///
    /// # Arguments
    ///
    /// * `chunk_information` - The metadata to write.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the metadata was written successfully.
    /// * `Err(eyre::Report)` if an error occurs.
    ///
    /// # Errors
    ///
    /// Returns an error if the metadata cannot be written due to I/O issues.
    pub async fn write_chunk_information(
        &self,
        chunk_information: &PoolChunkInformation,
    ) -> Result<()> {
        let filename = self.chunk_path_information();
        let file = File::create(&filename).await?;
        let mut file = tokio::io::BufWriter::new(file);

        let buffer = chunk_information.encode_length_delimited_to_vec();

        file.write_all(&buffer).await?;
        file.shutdown().await?;

        Ok(())
    }

    /// Reads metadata for the chunk from a file.
    ///
    /// # Returns
    ///
    /// * `Ok(PoolChunkInformation)` if the metadata was read successfully.
    /// * `Err(eyre::Report)` if an error occurs.
    ///
    /// # Errors
    ///
    /// Returns an error if the metadata cannot be read due to I/O issues.
    pub async fn chunk_information(&self) -> Result<PoolChunkInformation> {
        let filename = self.chunk_path_information();
        let file = File::open(&filename).await?;
        let mut file = tokio::io::BufReader::new(file);

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        let chunk_information = PoolChunkInformation::decode_length_delimited(&*buffer)?;

        Ok(chunk_information)
    }

    /// Calculates the hash of the chunk file.
    ///
    /// # Arguments
    ///
    /// * `chunk_algorithm` - The algorithm to use for hash calculation.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or the hash calculation fails.
    async fn calculate_chunk_hash(&self, chunk_algorithm: ChunkAlgorithm) -> Result<Vec<u8>> {
        let file = File::open(self.chunk_path()).await?;
        let file = tokio::io::BufReader::new(file);
        let mut file = WoodstockCompressionReader::new(file);
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

    /// Verifies the integrity of the chunk by comparing its hash.
    ///
    /// # Arguments
    ///
    /// * `chunk_algorith` - The algorithm used to calculate the hash.
    ///
    /// # Returns
    ///
    /// * `Ok(bool)` - `true` if the hash matches, `false` otherwise.
    /// * `Err(eyre::Report)` if an error occurs.
    ///
    /// # Errors
    ///
    /// Returns an error if the metadata does not match the file or if I/O issues occur.
    pub async fn check_chunk_information(&self, chunk_algorithm: ChunkAlgorithm) -> Result<bool> {
        debug!(
            "Checking chunk information for {:?} with algorithm {:?}",
            self.chunk_path(),
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

    /// Creates a writer for the chunk.
    ///
    /// # Arguments
    ///
    /// * `chunk_algorithm` - The algorithm used for chunk compression and hashing.
    ///
    /// # Returns
    ///
    /// * `Ok(PoolChunkWriter)` if the writer was created successfully.
    /// * `Err(eyre::Report)` if an error occurs.
    ///
    /// # Errors
    ///
    /// Returns an error if the writer cannot be created due to I/O issues.
    pub async fn writer(
        &self,
        chunk_algorithm: ChunkAlgorithm,
        compression_format: CompressionFormat,
    ) -> Result<PoolChunkWriter> {
        let pool_path = self.pool_path.clone();
        PoolChunkWriter::new(&pool_path, chunk_algorithm, compression_format).await
    }

    /// Writes data to the chunk and finalizes it.
    ///
    /// # Arguments
    ///
    /// * `data` - A stream of data to write to the chunk.
    /// * `debug_filename` - Debug information for the chunk.
    /// * `chunk_algorithm` - The algorithm used for chunk compression and hashing.
    ///
    /// # Returns
    ///
    /// * `Ok(PoolChunkInformation)` if the chunk was written successfully.
    /// * `Err(eyre::Report)` if an error occurs.
    ///
    /// # Errors
    ///
    /// Returns an error if the data cannot be written due to I/O issues.
    pub async fn write(
        &mut self,
        data: impl Stream<Item = Result<Vec<u8>>>,
        debug_filename: &[u8],
        chunk_algorithm: ChunkAlgorithm,
        compression_format: CompressionFormat,
    ) -> Result<PoolChunkInformation> {
        let mut writer = self.writer(chunk_algorithm, compression_format).await?;

        pin_mut!(data);

        // Write the data
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
        let chunk_information = writer
            .shutdown(self, debug_filename, compression_format)
            .await?;

        Ok(chunk_information)
    }

    /// Copies the chunk to a new location.
    ///
    /// # Arguments
    ///
    /// * `target_chunk` - The target chunk wrapper.
    /// * `chunk_algorithm` - The algorithm used for chunk compression and hashing.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the chunk was copied successfully.
    /// * `Err(eyre::Report)` if an error occurs.
    ///
    /// # Errors
    ///
    /// Returns an error if the copy operation fails due to I/O issues.
    pub async fn copy(
        &self,
        target_chunk: &mut PoolChunkWrapper,
        chunk_algorithm: ChunkAlgorithm,
    ) -> Result<()> {
        // Calculate the hash of the chunk
        let hash = self.calculate_chunk_hash(chunk_algorithm).await?;
        target_chunk.set_hash(Some(&hash));

        // Copy the chunk
        let target_chunk_path = target_chunk.chunk_path();
        if let Some(path) = target_chunk_path.parent() {
            create_dir_all(path).await?;
        }

        if target_chunk_path.exists() {
            warn!(
                "The target chunk already exists, possible risk of collision: {:?}",
                target_chunk_path.display()
            );
        }

        let reflink_result = reflink(self.chunk_path(), &target_chunk_path);
        if reflink_result.is_err() {
            copy(self.chunk_path(), &target_chunk_path).await?;
        }

        // Get the chunk information
        let mut chunk_information = self.chunk_information().await?;
        chunk_information.sha256.clone_from(&hash);

        // Set the hash of the target chunk

        let chunk_path = target_chunk.chunk_path();
        if let Some(path) = chunk_path.parent() {
            create_dir_all(path).await?;
        }

        target_chunk
            .write_chunk_information(&chunk_information)
            .await?;

        Ok(())
    }
}

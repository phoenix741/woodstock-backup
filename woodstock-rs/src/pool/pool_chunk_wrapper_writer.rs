use log::{debug, error, warn};
use std::path::{Path, PathBuf};
use tokio::fs::{create_dir_all, metadata, rename, File};
use tokio::io::AsyncWriteExt;

use crate::config::CHUNK_SIZE;
use crate::utils::chunk_hasher::{create_chunk_hasher, ChunkHasher};
use crate::utils::compression::{CompressionFormat, WoodstockCompressionWriter};
use crate::utils::path::vec_to_path;
use crate::ChunkAlgorithm;
use eyre::Result;

use super::{get_temp_chunk_path, PoolChunkInformation, PoolChunkWrapper};

/// # Pool Chunk Writer Module
///
/// This module provides the [`PoolChunkWriter`] struct and associated methods for writing
/// compressed chunk files in the Woodstock backup pool. It handles chunk creation, hashing,
/// metadata, and atomic file operations.
///
/// ## Main Structure
///
/// - [`PoolChunkWriter`]: Async writer for compressed chunk files.
///
/// ## Main Methods
///
/// - [`PoolChunkWriter::new`]: Create a new writer for a chunk.
/// - [`PoolChunkWriter::write`]: Write data to the chunk.
/// - [`PoolChunkWriter::shutdown`]: Finalize the chunk, compute hash, and move to final location.
///
/// ## Error Handling & Panics
///
/// - All async methods return `Result` and propagate errors using the `eyre` crate.
/// - Panics are not expected under normal operation.
///
/// ## See Also
///
/// - [`PoolChunkWrapper`], [`PoolChunkInformation`]: For chunk management and metadata
pub struct PoolChunkWriter {
    /// The compressed file being written.
    file: WoodstockCompressionWriter<File>,

    /// The uncompressed size of the data being written.
    uncompressed_size: usize,

    /// Optional hasher for calculating the file's hash.
    file_hasher: Option<Box<dyn ChunkHasher + Send + Sync>>, // Updated to use Box<dyn ChunkHasher>

    /// The temporary filename used during writing.
    tempfilename: PathBuf,
}

impl PoolChunkWriter {
    /// Create a new writer for a chunk.
    ///
    /// # Parameters
    ///
    /// - `pool_path`: The path to the pool directory.
    /// - `algorithm`: The chunk hashing algorithm.
    ///
    /// # Returns
    ///
    /// A new instance of `PoolChunkWriter`.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or the directory cannot be created.
    pub async fn new(
        pool_path: &Path,
        algorithm: ChunkAlgorithm,
        compression_format: CompressionFormat,
    ) -> Result<PoolChunkWriter> {
        let tempfilename = get_temp_chunk_path(pool_path);
        if let Some(path) = tempfilename.parent() {
            create_dir_all(path).await?;
        }

        let file = File::create(&tempfilename).await?;
        let file = tokio::io::BufWriter::new(file);
        let file = WoodstockCompressionWriter::new(file, compression_format);

        Ok(PoolChunkWriter {
            file,
            uncompressed_size: 0,
            file_hasher: Some(create_chunk_hasher(algorithm)),

            tempfilename,
        })
    }

    /// Write data to the chunk.
    ///
    /// # Parameters
    ///
    /// - `chunk`: The data to write.
    ///
    /// # Returns
    ///
    /// An empty result on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the data cannot be written.
    pub async fn write(&mut self, chunk: &[u8]) -> Result<()> {
        self.uncompressed_size += chunk.len();

        self.file.write_all(chunk).await?;
        if let Some(ref mut file_hasher) = self.file_hasher {
            file_hasher.update(chunk);
        };

        Ok(())
    }

    /// Finalizes the writing process and shuts down the writer.
    ///
    /// # Arguments
    /// * `wrapper` - The chunk wrapper associated with the file.
    /// * `debug_filename` - A debug identifier for the file.
    ///
    /// # Panics
    ///
    /// This function will panic if the `file_hasher` is `None` when accessed.
    /// Ensure that the hasher is properly initialized before calling this function.
    ///
    /// # Errors
    ///
    /// Returns an error if the shutdown process fails due to I/O issues.
    pub async fn shutdown(
        &mut self,
        wrapper: &mut PoolChunkWrapper,
        debug_filename: &[u8],
        compression_format: CompressionFormat,
    ) -> Result<PoolChunkInformation> {
        self.file.shutdown().await?;

        let mut file_hasher = self.file_hasher.take().unwrap();
        let file_hash: Vec<u8> = file_hasher.finalize();

        // Add a control
        if self.uncompressed_size > CHUNK_SIZE {
            if let Some(hash) = &wrapper.get_hash_str() {
                error!(
                    "Chunk {hash} has not the right size length {}",
                    self.uncompressed_size
                );
            }
        }

        if let Some(hash) = &wrapper.get_hash() {
            if hash.ne(&file_hash) {
                error!(
                    "When writing the chunk (for file {:?}), the hash should be {} but is {}",
                    vec_to_path(debug_filename),
                    hex::encode(hash),
                    hex::encode(&file_hash)
                );
            }
        }

        let metadata = metadata(&self.tempfilename).await?;

        wrapper.set_hash(Some(&file_hash));

        if wrapper.exists() {
            debug!("Chunk {:?} already exists", vec_to_path(debug_filename));
            let chunk_information = wrapper.chunk_information().await?;

            // Warn if size is different
            if chunk_information.size != u64::try_from(self.uncompressed_size)? {
                warn!(
                    "Chunk {:?} has not the right size length {}",
                    vec_to_path(debug_filename),
                    self.uncompressed_size
                );
            }

            // Return information of existing chunk
            Ok(chunk_information)
        } else {
            let chunk_information = PoolChunkInformation {
                size: u64::try_from(self.uncompressed_size)?,
                compressed_size: metadata.len(),
                sha256: file_hash.clone(),
                format: compression_format as u32,
            };

            let chunk_path = wrapper.chunk_path();
            if let Some(path) = chunk_path.parent() {
                create_dir_all(path).await?;
            }

            wrapper.write_chunk_information(&chunk_information).await?;
            rename(&self.tempfilename, chunk_path).await?;
            Ok(chunk_information)
        }
    }
}

impl Drop for PoolChunkWriter {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.tempfilename);
    }
}

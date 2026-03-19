//! # File Manifest Reader Module
//!
//! This module provides asynchronous reading and streaming of file manifests from the Woodstock backup pool.
//! It supports chunked decompression, random access, and hash calculation for deduplicated backup files.
//!
//! ## Main Structures
//!
//! - [`FileManifestReaderState`]: Holds state for reading a file manifest from the pool.
//!
//! ## Main Methods
//!
//! - [`FileManifestReaderState::new`]: Create a new reader state for a manifest.
//! - [`FileManifestReaderState::load_chunk`]: Load and decompress a chunk as needed.
//! - [`FileManifestReaderState::read_chunk`]: Read data from the current chunk into a buffer.
//! - [`FileManifest::open_from_pool`]: Open an async reader for a file from the pool.
//! - [`FileManifest::calculate_hash`]: Calculate the hash of a file's contents using a chunk algorithm.
//!
//! ## Error Handling & Panics
//!
//! - All public methods return `Result` and propagate I/O or decompression errors using the `eyre` crate.
//! - Panics are not expected under normal operation; errors are returned as `Result`.
//!
//! ## Generics
//!
//! - The `calculate_hash` method is generic over the chunk algorithm.
//!
//! ## See Also
//!
//! - [`FileManifest`], [`ChunkAlgorithm`], [`PoolChunkWrapper`]

use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

use bytes::Bytes;
use eyre::Result;
use futures::{pin_mut, stream::unfold};

use tokio::io::{AsyncBufRead, AsyncReadExt};
use tokio_util::io::StreamReader;
use tracing::warn;

use crate::{
    config::{BUFFER_SIZE, CHUNK_SIZE},
    pool::PoolChunkWrapper,
    utils::{chunk_hasher::create_chunk_hasher, compression::WoodstockCompressionReader},
    ChunkAlgorithm, FileManifest,
};

/// Represents the state for reading a file manifest from the pool.
///
/// This struct holds the current position, chunk information, and handles decompression.
struct FileManifestReaderState {
    /// Path to the file being read.
    file_path: PathBuf,
    /// List of chunk hashes for the file.
    chunks: Vec<Vec<u8>>,

    /// Path to the pool directory.
    pool_path: PathBuf,
    /// Current read position in the file.
    position: usize,

    /// The number of the currently loaded chunk.
    current_chunk_number: usize,
    /// The currently loaded chunk decoder, if any.
    current_chunk:
        Option<WoodstockCompressionReader<Box<dyn tokio::io::AsyncRead + Send + Sync + Unpin>>>,
}

impl FileManifestReaderState {
    /// Creates a new `FileManifestReaderState` for the given pool and manifest.
    ///
    /// # Arguments
    /// * `pool_path` - Path to the pool directory.
    /// * `manifest` - The file manifest to read.
    pub fn new(pool_path: &Path, manifest: &FileManifest) -> Self {
        Self {
            file_path: manifest.path().clone(),
            chunks: manifest.chunks.clone(),

            pool_path: pool_path.to_path_buf(),
            position: 0,

            current_chunk_number: 0,
            current_chunk: None,
        }
    }

    /// Returns the current chunk number based on the read position.
    fn get_chunk_number(&self) -> usize {
        self.position / CHUNK_SIZE
    }

    /// Returns a reference to the active chunk, if available.
    fn active_chunk(&self) -> Option<&Vec<u8>> {
        self.chunks.get(self.get_chunk_number())
    }

    /// Opens and returns a decoder for the given chunk.
    ///
    /// # Arguments
    /// * `active_chunk` - The chunk hash to open.
    ///
    /// # Errors
    /// Returns an error if the chunk file cannot be opened or decompressed.
    async fn open_chunk(
        &self,
        active_chunk: &Vec<u8>,
    ) -> Result<WoodstockCompressionReader<Box<dyn tokio::io::AsyncRead + Send + Sync + Unpin>>>
    {
        let chunk = PoolChunkWrapper::new(&self.pool_path, Some(active_chunk));
        chunk.open_chunk_reader().await
    }

    /// Loads the current chunk if needed and returns a mutable reference to the decoder.
    ///
    /// # Errors
    /// Returns an error if the chunk cannot be loaded or decompressed.
    async fn load_chunk(
        &mut self,
    ) -> Result<&mut WoodstockCompressionReader<Box<dyn tokio::io::AsyncRead + Send + Sync + Unpin>>>
    {
        if self.current_chunk.is_none() || self.current_chunk_number != self.get_chunk_number() {
            let Some(active_chunk) = self.active_chunk() else {
                panic!("Chunk doesn't exist");
            };

            let reader = self.open_chunk(active_chunk).await?;
            self.current_chunk = Some(reader);
            self.current_chunk_number = self.get_chunk_number();
        }
        // SAFETY: current_chunk is Some — set just above, or was already Some from a prior iteration
        Ok(self.current_chunk.as_mut().unwrap())
    }

    /// Reads data from the current chunk into the provided buffer.
    ///
    /// # Arguments
    /// * `buf` - The buffer to fill with data.
    ///
    /// # Errors
    /// Returns an error if reading from the chunk fails.
    async fn read_chunk(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let Some(active_chunk) = self.active_chunk() else {
            return Ok(0);
        };
        if active_chunk.is_empty() {
            warn!(
                "Corrupted chunk {} for file {:?}",
                self.get_chunk_number(),
                self.file_path
            );
            return Ok(0);
        }

        let reader = self
            .load_chunk()
            .await
            .map_err(|err| std::io::Error::new(ErrorKind::InvalidData, format!("{err}")))?;

        let size = reader.read(buf).await?;
        self.position += size;

        Ok(size)
    }
}

impl FileManifest {
    /// Opens a file from the pool and returns an async reader over its contents.
    ///
    /// # Arguments
    /// * `pool_path` - Path to the pool directory.
    ///
    /// # Returns
    /// An async buffered reader for the file's contents.
    #[must_use]
    pub fn open_from_pool(&self, pool_path: &Path) -> impl AsyncBufRead {
        let state = FileManifestReaderState::new(pool_path, self);

        let stream = unfold(state, |mut state| async move {
            let mut buffer = vec![0; BUFFER_SIZE];
            let size = state.read_chunk(&mut buffer).await;
            match size {
                Ok(size) => {
                    if size == 0 {
                        return None;
                    }

                    buffer.truncate(size);
                    Some((Ok(Bytes::from(buffer)), state))
                }
                Err(err) => Some((Err(err), state)),
            }
        });

        StreamReader::new(stream)
    }

    /// Calculates the hash of the file's contents using the specified chunk algorithm.
    ///
    /// # Arguments
    /// * `pool_path` - Path to the pool directory.
    /// * `chunk_algorithm` - The chunking algorithm to use for hashing.
    ///
    /// # Errors
    /// Returns an error if reading or hashing fails.
    pub async fn calculate_hash(
        &mut self,
        pool_path: &Path,
        chunk_algorithm: ChunkAlgorithm,
    ) -> Result<Vec<u8>> {
        let mut hasher = create_chunk_hasher(chunk_algorithm);
        let reader = self.open_from_pool(pool_path);
        pin_mut!(reader);

        loop {
            let mut buf = vec![0; BUFFER_SIZE];
            let size = reader.read(&mut buf).await?;
            if size == 0 {
                break;
            }

            hasher.update(&buf[..size]);
        }

        Ok(hasher.finalize())
    }
}

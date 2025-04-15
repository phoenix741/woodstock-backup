use async_compression::tokio::bufread::ZlibDecoder;
use eyre::Result;
use futures::{pin_mut, Stream, StreamExt};
use log::{error, warn};
use prost::Message;
use std::path::{Path, PathBuf};
use tokio::{
    fs::{copy, create_dir_all, remove_file, rename, File},
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::config::BUFFER_SIZE;
use crate::utils::chunk_hasher::create_chunk_hasher;
use crate::ChunkAlgorithm;
use reflink_copy::reflink;

use super::{
    calculate_chunk_path, get_temp_chunk_path, pool_chunk_information::PoolChunkInformation,
    PoolChunkWriter,
};

pub struct PoolChunkWrapper {
    pool_path: PathBuf,
    hash_str: Option<String>,
    hash: Option<Vec<u8>>,
}

impl PoolChunkWrapper {
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
        self.chunk_path().exists()
    }

    pub async fn remove(&self) -> std::io::Result<()> {
        remove_file(self.chunk_path()).await
    }

    pub async fn mv<P: AsRef<Path>>(&self, target_path: P) -> std::io::Result<()> {
        assert_ne!(self.hash_str, None, "Hash of the file shouldn't be None");

        if rename(self.chunk_path(), &target_path).await.is_err() {
            // copy the file if the rename fails
            copy(self.chunk_path(), &target_path).await?;
            self.remove().await?;
        }

        Ok(())
    }

    #[must_use]
    pub fn chunk_path(&self) -> PathBuf {
        match &self.hash_str {
            Some(hash) => calculate_chunk_path(&self.pool_path, hash),
            None => get_temp_chunk_path(&self.pool_path),
        }
    }

    fn chunk_path_information(&self) -> PathBuf {
        self.chunk_path().with_extension("info")
    }

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

    pub async fn chunk_information(&self) -> Result<PoolChunkInformation> {
        let filename = self.chunk_path_information();
        let file = File::open(&filename).await?;
        let mut file = tokio::io::BufReader::new(file);

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        let chunk_information = PoolChunkInformation::decode_length_delimited(&*buffer)?;

        Ok(chunk_information)
    }

    async fn calculate_chunk_hash(&self, chunk_algorithm: &ChunkAlgorithm) -> Result<Vec<u8>> {
        let file = File::open(self.chunk_path()).await?;
        let file = tokio::io::BufReader::new(file);
        let mut file = ZlibDecoder::new(file);
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

    pub async fn check_chunk_information(&self, chunk_algorith: &ChunkAlgorithm) -> Result<bool> {
        let file_hash = self.calculate_chunk_hash(chunk_algorith).await?;

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

    pub async fn writer(&self, chunk_algorithm: &ChunkAlgorithm) -> Result<PoolChunkWriter> {
        let pool_path = self.pool_path.clone();
        PoolChunkWriter::new(&pool_path, chunk_algorithm).await
    }

    pub async fn write(
        &mut self,
        data: impl Stream<Item = Result<Vec<u8>>>,
        debug_filename: &[u8],
        chunk_algorithm: &ChunkAlgorithm,
    ) -> Result<PoolChunkInformation> {
        let mut writer = self.writer(chunk_algorithm).await?;

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
        let chunk_information = writer.shutdown(self, debug_filename).await?;

        Ok(chunk_information)
    }

    pub async fn copy(
        &self,
        target_chunk: &mut PoolChunkWrapper,
        chunk_algorithm: &ChunkAlgorithm,
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
        chunk_information.sha256 = hash.clone();

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

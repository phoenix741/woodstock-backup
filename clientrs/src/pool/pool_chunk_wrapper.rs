use async_compression::tokio::bufread::ZlibDecoder;
use async_compression::tokio::write::ZlibEncoder;
use futures::{pin_mut, Stream, StreamExt};
use log::{debug, error};
use prost::Message;
use rand::Rng;
use sha2::Digest;
use sha3::Sha3_256;
use std::path::{Path, PathBuf};
use tokio::{
    fs::{copy, create_dir_all, metadata, remove_file, rename, File},
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::{
    scanner::{BUFFER_SIZE, CHUNK_SIZE},
    utils::path::vec_to_path,
};

use super::pool_chunk_information::PoolChunkInformation;

fn calculate_chunk_path(pool_path: &Path, chunk: &str) -> PathBuf {
    let part1 = &chunk[0..2];
    let part2 = &chunk[2..4];
    let part3 = &chunk[4..6];

    Path::new(pool_path)
        .join(part1)
        .join(part2)
        .join(part3)
        .join(chunk.to_string() + "-sha256.zz")
}

fn get_temp_chunk_path(pool_path: &Path) -> PathBuf {
    // Generate a string of 30 characters with random characters (base 36)
    let temporary_filename = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(30)
        .map(char::from)
        .collect::<String>();

    Path::new(pool_path).join("_new").join(temporary_filename)
}

struct DropTempFile {
    path: PathBuf,
}

impl Drop for DropTempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

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

    pub async fn mv(&self, target_path: &Path) -> std::io::Result<()> {
        assert_ne!(self.hash_str, None, "Hash of the file shouldn't be None");

        if rename(self.chunk_path(), target_path).await.is_err() {
            // copy the file if the rename fails
            copy(self.chunk_path(), target_path).await?;
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

    async fn write_chunk_information(
        &self,
        chunk_information: &PoolChunkInformation,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let filename = self.chunk_path_information();
        let file = File::create(&filename).await?;
        let mut file = tokio::io::BufWriter::new(file);

        let buffer = chunk_information.encode_length_delimited_to_vec();

        file.write_all(&buffer).await?;
        file.shutdown().await?;

        Ok(())
    }

    pub async fn chunk_information(
        &self,
    ) -> Result<PoolChunkInformation, Box<dyn std::error::Error>> {
        let filename = self.chunk_path_information();
        let file = File::open(&filename).await?;
        let mut file = tokio::io::BufReader::new(file);

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        let chunk_information = PoolChunkInformation::decode_length_delimited(&*buffer)?;

        Ok(chunk_information)
    }

    pub async fn check_chunk_information(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let file = File::open(self.chunk_path()).await?;
        let file = tokio::io::BufReader::new(file);
        let mut file = ZlibDecoder::new(file);
        let mut hasher = Sha3_256::new();

        let mut buffer = vec![0u8; BUFFER_SIZE];
        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        let file_hash = hasher.finalize().to_vec();

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

    pub async fn write(
        &mut self,
        data: impl Stream<Item = Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>>>,
        debug_filename: &[u8],
    ) -> Result<PoolChunkInformation, Box<dyn std::error::Error>> {
        let tempfilename = get_temp_chunk_path(&self.pool_path);
        if let Some(path) = tempfilename.parent() {
            create_dir_all(path).await?;
        }

        let _drop_temp_file = DropTempFile {
            path: tempfilename.clone(),
        };

        let file = File::create(&tempfilename).await?;
        let file = tokio::io::BufWriter::new(file);
        let mut file = ZlibEncoder::new(file);

        pin_mut!(data);

        // Write the data
        let mut uncompressed_size = 0;
        let mut file_hasher = Sha3_256::new();
        while let Some(chunk) = data.next().await {
            match chunk {
                Ok(chunk) => {
                    uncompressed_size += chunk.len();

                    file.write_all(&chunk).await?;
                    file_hasher.update(&chunk[..]);
                }
                Err(e) => {
                    error!("Error while reading the chunk: {:?}", e);
                    return Err(e);
                }
            };
        }
        file.shutdown().await?;
        let file_hash: Vec<u8> = file_hasher.finalize().to_vec();

        // Add a control
        if uncompressed_size > CHUNK_SIZE {
            if let Some(hash) = &self.hash_str {
                error!("Chunk {hash} has not the right size length {uncompressed_size}");
            }
        }

        if let Some(hash) = &self.hash {
            if hash.ne(&file_hash) {
                error!(
                    "When writing the chunk (for file {:?}), the hash should be {} but is {}",
                    vec_to_path(debug_filename),
                    hex::encode(hash),
                    hex::encode(&file_hash)
                );
            }
        }

        let metadata = metadata(&tempfilename).await?;
        let chunk_information = PoolChunkInformation {
            size: u64::try_from(uncompressed_size)?,
            compressed_size: metadata.len(),
            sha256: file_hash.clone(),
        };

        self.set_hash(Some(&file_hash));

        if self.exists() {
            debug!("Chunk {:?} already exists", vec_to_path(debug_filename));
        } else {
            let chunk_path = self.chunk_path();
            if let Some(path) = chunk_path.parent() {
                create_dir_all(path).await?;
            }

            self.write_chunk_information(&chunk_information).await?;
            rename(tempfilename, chunk_path).await?;
        }

        Ok(chunk_information)
    }
}

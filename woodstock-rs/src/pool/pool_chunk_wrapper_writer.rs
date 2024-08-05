use async_compression::tokio::write::ZlibEncoder;
use log::{debug, error};
use sha2::digest::core_api::CoreWrapper;
use sha3::Digest;
use sha3::{Sha3_256, Sha3_256Core};
use std::path::{Path, PathBuf};
use tokio::fs::{create_dir_all, metadata, rename, File};
use tokio::io::AsyncWriteExt;

use crate::config::CHUNK_SIZE;
use crate::utils::path::vec_to_path;
use eyre::Result;

use super::{get_temp_chunk_path, PoolChunkInformation, PoolChunkWrapper};

/// The writer is an `AsyncWriter`
pub struct PoolChunkWriter {
    file: ZlibEncoder<tokio::io::BufWriter<File>>,

    uncompressed_size: usize,
    file_hasher: Option<CoreWrapper<Sha3_256Core>>,

    tempfilename: PathBuf,
}

impl PoolChunkWriter {
    pub async fn new(pool_path: &Path) -> Result<PoolChunkWriter> {
        let tempfilename = get_temp_chunk_path(pool_path);
        if let Some(path) = tempfilename.parent() {
            create_dir_all(path).await?;
        }

        let file = File::create(&tempfilename).await?;
        let file = tokio::io::BufWriter::new(file);
        let file = ZlibEncoder::new(file);

        Ok(PoolChunkWriter {
            file,
            uncompressed_size: 0,
            file_hasher: Some(Sha3_256::new()),

            tempfilename,
        })
    }

    pub async fn write(&mut self, chunk: &[u8]) -> Result<()> {
        self.uncompressed_size += chunk.len();

        self.file.write_all(chunk).await?;
        if let Some(ref mut file_hasher) = self.file_hasher {
            file_hasher.update(chunk);
        };

        Ok(())
    }

    pub async fn shutdown(
        &mut self,
        wrapper: &mut PoolChunkWrapper,
        debug_filename: &[u8],
    ) -> Result<PoolChunkInformation> {
        self.file.shutdown().await?;

        let file_hasher = self.file_hasher.take().unwrap();
        let file_hash: Vec<u8> = file_hasher.finalize().to_vec();

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
        let chunk_information = PoolChunkInformation {
            size: u64::try_from(self.uncompressed_size)?,
            compressed_size: metadata.len(),
            sha256: file_hash.clone(),
        };

        wrapper.set_hash(Some(&file_hash));

        if wrapper.exists() {
            debug!("Chunk {:?} already exists", vec_to_path(debug_filename));
        } else {
            let chunk_path = wrapper.chunk_path();
            if let Some(path) = chunk_path.parent() {
                create_dir_all(path).await?;
            }

            wrapper.write_chunk_information(&chunk_information).await?;
            rename(&self.tempfilename, chunk_path).await?;
        }
        Ok(chunk_information)
    }
}

impl Drop for PoolChunkWriter {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.tempfilename);
    }
}

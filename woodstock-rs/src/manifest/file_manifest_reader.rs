use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

use async_compression::tokio::bufread::ZlibDecoder;
use bytes::Bytes;
use eyre::Result;
use futures::{pin_mut, stream::unfold};

use log::warn;
use sha2::Digest;
use sha3::Sha3_256;
use tokio::{
    fs::File,
    io::{AsyncBufRead, AsyncReadExt, BufReader},
};
use tokio_util::io::StreamReader;

use crate::{
    config::{BUFFER_SIZE, CHUNK_SIZE},
    pool::PoolChunkWrapper,
    FileManifest,
};

struct FileManifestReaderState<'manifest> {
    manifest: &'manifest FileManifest,
    pool_path: PathBuf,
    position: usize,

    current_chunk_number: usize,
    current_chunk: Option<ZlibDecoder<BufReader<File>>>,
}

impl<'manifest> FileManifestReaderState<'manifest> {
    pub fn new(pool_path: &Path, manifest: &'manifest FileManifest) -> Self {
        Self {
            pool_path: pool_path.to_path_buf(),
            manifest,
            position: 0,
            current_chunk_number: 0,
            current_chunk: None,
        }
    }

    fn get_chunk_number(&self) -> usize {
        self.position / CHUNK_SIZE
    }

    fn active_chunk(&self) -> Option<&Vec<u8>> {
        self.manifest.chunks.get(self.get_chunk_number())
    }

    async fn open_chunk(&self, active_chunk: &Vec<u8>) -> Result<ZlibDecoder<BufReader<File>>> {
        let chunk = PoolChunkWrapper::new(&self.pool_path, Some(active_chunk));

        let chunk_path = chunk.chunk_path();

        // Read all the chunk content
        let file = File::open(chunk_path).await?;
        let file = BufReader::new(file);
        let file = ZlibDecoder::new(file);

        Ok(file)
    }

    async fn load_chunk(&mut self) -> Result<&mut ZlibDecoder<BufReader<File>>> {
        if self.current_chunk.is_none() || self.current_chunk_number != self.get_chunk_number() {
            let Some(active_chunk) = self.active_chunk() else {
                panic!("Chunk doesn't exist");
            };

            let reader = self.open_chunk(active_chunk).await?;
            self.current_chunk = Some(reader);
            self.current_chunk_number = self.get_chunk_number();
        }

        Ok(self.current_chunk.as_mut().unwrap())
    }

    async fn read_chunk(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let Some(active_chunk) = self.active_chunk() else {
            return Ok(0);
        };
        if active_chunk.is_empty() {
            warn!(
                "Corrupted chunk {} for file {:?}",
                self.get_chunk_number(),
                self.manifest.path()
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
    #[must_use] pub fn open_from_pool(&self, pool_path: &Path) -> impl AsyncBufRead + '_ {
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

    pub async fn calculate_hash(&mut self, pool_path: &Path) -> Result<Vec<u8>> {
        let mut hasher = Sha3_256::new();
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

        Ok(hasher.finalize().to_vec())
    }
}

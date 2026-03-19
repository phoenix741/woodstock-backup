//! Pool V3 per-backup staging file helpers.
//!
//! A staging file belongs to one backup directory and stores one append-only
//! sequence of chunk reference deltas used for crash-safe publication.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use eyre::Result;
use tokio::fs::File;
use tokio::io::BufReader;

use crate::pool::{
    PoolV3StagingChunkRecord, PoolV3StagingEntry, PoolV3StagingEnvelope, PoolV3StagingHeader,
};
use crate::proto::{read_optional_length_delimited_message, ProtobufWriter, UnCompressedWriter};

/// Append-only staging file used during one Pool V3 backup publication.
pub struct PoolV3StagingFile {
    path: PathBuf,
}

pub struct PoolV3StagingWriter {
    writer: ProtobufWriter<UnCompressedWriter, PoolV3StagingEnvelope>,
}

impl PoolV3StagingFile {
    #[must_use]
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub async fn create_or_open_writer(
        &self,
        hostname: &str,
        backup_id: &[u8],
    ) -> Result<PoolV3StagingWriter> {
        if self.path.exists() {
            let writer =
                ProtobufWriter::<UnCompressedWriter, PoolV3StagingEnvelope>::open(&self.path)
                    .await?;
            return Ok(PoolV3StagingWriter { writer });
        }

        let mut writer =
            ProtobufWriter::<UnCompressedWriter, PoolV3StagingEnvelope>::new(&self.path, false)
                .await?;
        let header = PoolV3StagingHeader {
            format_version: 1,
            hostname: hostname.to_string(),
            backup_id: backup_id.to_vec(),
            created_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        };
        writer
            .write(&PoolV3StagingEnvelope {
                header: Some(header),
                entry: None,
            })
            .await?;
        Ok(PoolV3StagingWriter { writer })
    }

    pub async fn append_chunk(&self, chunk: &PoolV3StagingChunkRecord) -> Result<()> {
        let mut writer =
            ProtobufWriter::<UnCompressedWriter, PoolV3StagingEnvelope>::open(&self.path).await?;
        writer
            .write(&PoolV3StagingEnvelope {
                header: None,
                entry: Some(PoolV3StagingEntry {
                    chunk: Some(chunk.clone()),
                }),
            })
            .await?;
        writer.flush().await?;

        Ok(())
    }

    pub async fn read_header(&self) -> Result<Option<PoolV3StagingHeader>> {
        if !self.path.exists() {
            return Ok(None);
        }

        let file = File::open(&self.path).await?;
        let mut file = BufReader::new(file);
        let mut buffer = Vec::with_capacity(256);
        let next_header: std::io::Result<Option<(PoolV3StagingEnvelope, usize)>> =
            read_optional_length_delimited_message::<PoolV3StagingEnvelope, _>(
                &mut file,
                &mut buffer,
            )
            .await;

        match next_header {
            Ok(Some((envelope, _))) => Ok(envelope.header),
            Ok(None) => Ok(None),
            Err(error) if error.kind() == ErrorKind::UnexpectedEof => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub async fn read_chunks(&self) -> Result<Vec<PoolV3StagingChunkRecord>> {
        let file = File::open(&self.path).await?;
        let mut file = BufReader::new(file);
        let mut buffer = Vec::with_capacity(256);
        let mut records: Vec<PoolV3StagingChunkRecord> = Vec::new();

        loop {
            let next_record: std::io::Result<Option<(PoolV3StagingEnvelope, usize)>> =
                read_optional_length_delimited_message::<PoolV3StagingEnvelope, _>(
                    &mut file,
                    &mut buffer,
                )
                .await;

            match next_record {
                Ok(Some((envelope, _))) => {
                    if let Some(entry) = envelope.entry {
                        if let Some(chunk) = entry.chunk {
                            records.push(chunk);
                        }
                    }
                }
                Ok(None) => break,
                Err(error) if error.kind() == ErrorKind::UnexpectedEof => break,
                Err(error) => return Err(error.into()),
            }
        }

        Ok(records)
    }
}

impl PoolV3StagingWriter {
    pub async fn append_chunk(&mut self, chunk: &PoolV3StagingChunkRecord) -> Result<()> {
        self.writer
            .write(&PoolV3StagingEnvelope {
                header: None,
                entry: Some(PoolV3StagingEntry {
                    chunk: Some(chunk.clone()),
                }),
            })
            .await
    }

    pub async fn flush(&mut self) -> Result<()> {
        self.writer.flush().await
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use uuid::Uuid;

    use super::*;

    #[tokio::test]
    async fn create_append_and_read_staging_file() {
        let tempdir = tempdir().unwrap();
        let staging = PoolV3StagingFile::new(tempdir.path().join("pool-v3.staging"));

        let backup_id = Uuid::new_v4();
        let mut writer = staging
            .create_or_open_writer("host-a", backup_id.as_bytes())
            .await
            .unwrap();
        writer
            .append_chunk(&PoolV3StagingChunkRecord {
                hash: vec![0xCD; 32],
                size: 4096,
                compressed_size: 1024,
                chunk_header_size: 28,
                compression_format: 2,
                ref_count_delta: 1,
                publishes_new_chunk: true,
                segment_id: 7,
                offset: 128,
            })
            .await
            .unwrap();
        writer.flush().await.unwrap();

        let header = staging.read_header().await.unwrap().unwrap();
        let records = staging.read_chunks().await.unwrap();

        assert_eq!(header.hostname, "host-a");
        assert_eq!(header.backup_id, backup_id.as_bytes());
        assert_eq!(records.len(), 1);
        assert!(matches!(
            &records[0],
            PoolV3StagingChunkRecord {
                segment_id: 7,
                publishes_new_chunk: true,
                ..
            }
        ));
    }
}

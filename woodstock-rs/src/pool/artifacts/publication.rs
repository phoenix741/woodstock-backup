//! Pool V3 per-backup publication artifact helpers.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use eyre::Result;
use tokio::fs::File;
use tokio::io::BufReader;

use crate::pool::{
    PoolV3PublicationChunkEntry, PoolV3PublicationEntry, PoolV3PublicationEnvelope,
    PoolV3PublicationHeader, PoolV3StagingChunkRecord,
};
use crate::proto::{read_optional_length_delimited_message, ProtobufWriter, UnCompressedWriter};

/// Persistent per-backup publication artifact used to replay positive reference deltas.
pub struct PoolV3PublicationFile {
    path: PathBuf,
}

impl PoolV3PublicationFile {
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

    pub async fn create_with_records(
        &self,
        hostname: &str,
        backup_id: &[u8],
        records: &[PoolV3StagingChunkRecord],
    ) -> Result<()> {
        let mut writer =
            ProtobufWriter::<UnCompressedWriter, PoolV3PublicationEnvelope>::new(&self.path, true)
                .await?;
        writer
            .write(&PoolV3PublicationEnvelope {
                header: Some(PoolV3PublicationHeader {
                    format_version: 1,
                    hostname: hostname.to_string(),
                    backup_id: backup_id.to_vec(),
                    created_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
                }),
                entry: None,
            })
            .await?;

        for record in records {
            writer
                .write(&PoolV3PublicationEnvelope {
                    header: None,
                    entry: Some(PoolV3PublicationEntry {
                        chunk: Some(PoolV3PublicationChunkEntry {
                            hash: record.hash.clone(),
                            ref_count_delta: record.ref_count_delta,
                            publishes_new_chunk: record.publishes_new_chunk,
                            segment_id: record.segment_id,
                            offset: record.offset,
                            size: record.size,
                            compressed_size: record.compressed_size,
                            chunk_header_size: record.chunk_header_size,
                            compression_format: record.compression_format,
                        }),
                    }),
                })
                .await?;
        }

        writer.flush().await?;
        Ok(())
    }

    pub async fn read_records(&self) -> Result<Vec<PoolV3PublicationChunkEntry>> {
        let file = File::open(&self.path).await?;
        let mut file = BufReader::new(file);
        let mut buffer = Vec::with_capacity(256);
        let mut records = Vec::new();

        loop {
            let next_record: std::io::Result<Option<(PoolV3PublicationEnvelope, usize)>> =
                read_optional_length_delimited_message::<PoolV3PublicationEnvelope, _>(
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

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use uuid::Uuid;

    use super::*;

    #[tokio::test]
    async fn create_and_read_publication_file() {
        let tempdir = tempdir().unwrap();
        let publication = PoolV3PublicationFile::new(tempdir.path().join("pool-v3.publication"));
        let backup_id = Uuid::now_v7();

        publication
            .create_with_records(
                "host-a",
                backup_id.as_bytes(),
                &[PoolV3StagingChunkRecord {
                    hash: vec![0xAB; 32],
                    size: 4096,
                    compressed_size: 1024,
                    chunk_header_size: 28,
                    compression_format: 2,
                    ref_count_delta: 2,
                    publishes_new_chunk: true,
                    segment_id: 7,
                    offset: 128,
                }],
            )
            .await
            .unwrap();

        let records = publication.read_records().await.unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].ref_count_delta, 2);
        assert_eq!(records[0].segment_id, 7);
    }
}

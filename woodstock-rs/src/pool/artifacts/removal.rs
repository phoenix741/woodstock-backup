//! Pool V3 per-backup removal artifact helpers.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use eyre::Result;
use tokio::fs::File;
use tokio::io::BufReader;

use crate::pool::PoolV3RemovalChunkRecord;
use crate::proto::{read_optional_length_delimited_message, ProtobufWriter, UnCompressedWriter};

/// Persistent per-backup removal artifact used to replay negative reference deltas.
pub struct PoolV3RemovalFile {
    path: PathBuf,
}

impl PoolV3RemovalFile {
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

    pub async fn create_with_records(&self, records: &[PoolV3RemovalChunkRecord]) -> Result<()> {
        let mut records_writer =
            ProtobufWriter::<UnCompressedWriter, PoolV3RemovalChunkRecord>::new(&self.path, true)
                .await?;
        for record in records {
            records_writer.write(record).await?;
        }
        records_writer.flush().await?;

        Ok(())
    }

    pub async fn read_records(&self) -> Result<Vec<PoolV3RemovalChunkRecord>> {
        let file = File::open(&self.path).await?;
        let mut file = BufReader::new(file);
        let mut buffer = Vec::with_capacity(256);
        let mut records: Vec<PoolV3RemovalChunkRecord> = Vec::new();
        loop {
            let next_record: std::io::Result<Option<(PoolV3RemovalChunkRecord, usize)>> =
                read_optional_length_delimited_message::<PoolV3RemovalChunkRecord, _>(
                    &mut file,
                    &mut buffer,
                )
                .await;

            match next_record {
                Ok(Some((record, _))) => records.push(record),
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

    use super::*;

    #[tokio::test]
    async fn create_and_read_removal_file() {
        let tempdir = tempdir().unwrap();
        let removal = PoolV3RemovalFile::new(tempdir.path().join("pool-v3.removal"));

        removal
            .create_with_records(&[PoolV3RemovalChunkRecord {
                hash: vec![0xCD; 32],
                size: 4096,
                compressed_size: 1024,
                chunk_header_size: 28,
                ref_count_delta: 3,
            }])
            .await
            .unwrap();

        let records = removal.read_records().await.unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].ref_count_delta, 3);
    }
}

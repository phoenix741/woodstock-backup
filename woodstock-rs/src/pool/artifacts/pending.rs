//! Pool V3 pending integration descriptor helpers.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use eyre::Result;
use tokio::fs::File;
use tokio::io::BufReader;

use crate::pool::PoolV3PendingHeader;
use crate::proto::{read_optional_length_delimited_message, ProtobufWriter, UnCompressedWriter};

/// Small pending descriptor pointing to a durable publication or removal artifact.
pub struct PoolV3PendingFile {
    path: PathBuf,
}

impl PoolV3PendingFile {
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

    pub async fn create(
        &self,
        operation_id: &str,
        operation_type: &str,
        hostname: &str,
        backup_id: &[u8],
        journal_path: &Path,
    ) -> Result<()> {
        let mut writer =
            ProtobufWriter::<UnCompressedWriter, PoolV3PendingHeader>::new(&self.path, true)
                .await?;
        writer
            .write(&PoolV3PendingHeader {
                format_version: 1,
                operation_id: operation_id.to_string(),
                operation_type: operation_type.to_string(),
                hostname: hostname.to_string(),
                backup_id: backup_id.to_vec(),
                journal_path: journal_path.display().to_string(),
                created_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            })
            .await?;
        writer.flush().await?;
        Ok(())
    }

    pub async fn read_header(&self) -> Result<Option<PoolV3PendingHeader>> {
        if !self.path.exists() {
            return Ok(None);
        }

        let file = File::open(&self.path).await?;
        let mut file = BufReader::new(file);
        let mut buffer = Vec::with_capacity(256);
        Ok(
            read_optional_length_delimited_message::<PoolV3PendingHeader, _>(
                &mut file,
                &mut buffer,
            )
            .await?
            .map(|(header, _)| header),
        )
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use uuid::Uuid;

    use super::*;

    #[tokio::test]
    async fn create_and_read_pending_header() {
        let tempdir = tempdir().unwrap();
        let pending = PoolV3PendingFile::new(tempdir.path().join("publication-host-a"));
        let backup_id = Uuid::now_v7();

        pending
            .create(
                "publication-host-a",
                "publication",
                "host-a",
                backup_id.as_bytes(),
                &tempdir.path().join("pool-v3.publication"),
            )
            .await
            .unwrap();

        let header = pending.read_header().await.unwrap().unwrap();
        assert_eq!(header.operation_type, "publication");
        assert_eq!(header.hostname, "host-a");
    }
}

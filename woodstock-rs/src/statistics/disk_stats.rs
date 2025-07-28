//! Historisation de l'usage disque brut (type FS, taille, utilisé, libre) similaire à DiskStatisticsService TS.
use std::{io::ErrorKind, path::Path};

use chrono::Local;
use eyre::Result;
use tokio::fs::write;
use tracing::error;

use super::{HistoricalDiskStatistics, StatsDiskUsage};

const DISK_HISTORY_FILE: &str = "disk_history.yml"; // aligné TS

pub async fn read_disk_history(dirname: &Path) -> Vec<HistoricalDiskStatistics> {
    let filename = dirname.join(DISK_HISTORY_FILE);
    let yaml = tokio::fs::read_to_string(filename).await;

    match yaml {
        Ok(yaml) => {
            let history = serde_yaml_ng::from_str(&yaml);
            match history {
                Ok(history) => history,
                Err(e) => {
                    error!("Failed to parse history: {e}");
                    Vec::new()
                }
            }
        }
        Err(e) => {
            if e.kind() != ErrorKind::NotFound {
                error!("Failed to read backups: {e}");
            }
            Vec::new()
        }
    }
}

pub async fn append_disk_history(dirname: &Path, usage: &StatsDiskUsage) -> Result<()> {
    let mut history = read_disk_history(dirname).await;

    let date = Local::now();

    history.push(HistoricalDiskStatistics {
        fstype: usage.fstype.clone(),
        size: usage.size,
        used: usage.used,
        free: usage.free,
        date,
    });

    let yaml = serde_yaml_ng::to_string(&history)?;
    let filename = dirname.join(DISK_HISTORY_FILE);
    write(filename, yaml).await?;

    Ok(())
}

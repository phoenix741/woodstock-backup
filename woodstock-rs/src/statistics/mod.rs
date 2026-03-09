//! Statistics module.
//!
//! This module provides types and functions for managing and persisting backup pool statistics and history.

pub mod disk_stats;
pub mod instant_stats;
/// Contains models for statistics data.
mod model;

use std::{io::ErrorKind, path::Path};
use tracing::error;

use chrono::{DateTime, Local};
pub use model::{
    HistoricalDiskStatistics, HistoricalPoolStatistics, HostStatsUsage, PoolStatistics,
    StatsDiskUsage, WithDate,
};
use tokio::fs::write;

use eyre::Result;

/// Reads the pool statistics from a YAML file.
///
/// # Arguments
/// * `dirname` - The directory containing the statistics file.
///
/// # Returns
///
/// * `Ok(PoolStatistics)` if the statistics are successfully read.
/// * `Err(eyre::Report)` if an error occurs during reading.
///
/// # Errors
///
/// Returns an error if the statistics file cannot be read or deserialized from YAML.
pub async fn read_statistics(dirname: &Path) -> PoolStatistics {
    // Deserialize PoolStatistics from yaml format
    let filename = dirname.join("statistics.yml");
    let yaml = tokio::fs::read_to_string(filename)
        .await
        .unwrap_or_default();
    serde_yaml_ng::from_str(&yaml).unwrap_or_default()
}

/// Writes the pool statistics to a YAML file and appends them to the history.
///
/// # Arguments
/// * `statistics` - The pool statistics to write.
/// * `dirname` - The directory to write the statistics file to.
/// * `date` - The date of the statistics.
///
/// # Returns
///
/// * `Ok(())` if the statistics are successfully written.
/// * `Err(eyre::Report)` if an error occurs during writing.
///
/// # Errors
///
/// Returns an error if the statistics file cannot be written or if appending to the history fails.
pub async fn write_statistics(
    statistics: &PoolStatistics,
    dirname: &Path,
    date: &DateTime<Local>,
) -> Result<()> {
    // Serialize PoolStatistics in yaml format
    let filename = dirname.join("statistics.yml");
    let yaml = serde_yaml_ng::to_string(statistics)?;
    write(filename, yaml).await?;

    append_history_to_statistics(statistics, dirname, date).await?;

    Ok(())
}

/// Loads the history of pool statistics from a YAML file.
///
/// # Arguments
/// * `dirname` - The directory containing the history file.
///
/// # Returns
///
/// * `Ok(Vec<HistoricalPoolStatistics>)` if the history is successfully loaded.
/// * `Err(eyre::Report)` if an error occurs during loading.
///
/// # Errors
///
/// Returns an error if the history file cannot be read or deserialized from YAML.
pub async fn load_history(dirname: &Path) -> Vec<HistoricalPoolStatistics> {
    // Deserialize PoolStatistics from yaml format
    let filename = dirname.join("history.yml");
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

/// Appends the current pool statistics to the history file.
///
/// # Arguments
/// * `statistics` - The pool statistics to append.
/// * `dirname` - The directory containing the history file.
/// * `date` - The date of the statistics.
///
/// # Returns
///
/// * `Ok(())` if the statistics are successfully appended.
/// * `Err(eyre::Report)` if an error occurs during appending.
///
/// # Errors
///
/// Returns an error if the history file cannot be read, written, or if serialization fails.
pub async fn append_history_to_statistics(
    statistics: &PoolStatistics,
    dirname: &Path,
    date: &DateTime<Local>,
) -> Result<()> {
    let mut histories = load_history(dirname).await;

    let history = HistoricalPoolStatistics {
        date: *date,
        longest_chain: statistics.longest_chain,
        nb_chunk: statistics.nb_chunk,
        nb_ref: statistics.nb_ref,
        size: statistics.size,
        compressed_size: statistics.compressed_size,
        unused_size: statistics.unused_size,
    };

    histories.push(history);
    // TODO: To Remove ; Sort history by dates (normally useless)
    histories.sort_by(|a, b| a.date.cmp(&b.date));

    // Serialize PoolStatistics in yaml format
    let filename = dirname.join("history.yml");
    let yaml = serde_yaml_ng::to_string(&histories)?;
    write(filename, yaml).await?;

    Ok(())
}

mod statistics;

use std::{path::Path, time::SystemTime};

pub use statistics::*;
use tokio::fs::write;

pub async fn read_statistics(dirname: &Path) -> Result<PoolStatistics, Box<dyn std::error::Error>> {
    // Deserialize PoolStatistics from yaml format
    let filename = dirname.join("statistics.yml");
    let yaml = tokio::fs::read_to_string(filename).await?;
    let statistics = serde_yaml::from_str(&yaml)?;

    Ok(statistics)
}

pub async fn write_statistics(
    statistics: &PoolStatistics,
    dirname: &Path,
    date: &SystemTime,
) -> Result<(), Box<dyn std::error::Error>> {
    // Serialize PoolStatistics in yaml format
    let filename = dirname.join("statistics.yml");
    let yaml = serde_yaml::to_string(statistics)?;
    write(filename, yaml).await?;

    append_history_to_statistics(statistics, dirname, date).await?;

    Ok(())
}

pub async fn load_history(
    dirname: &Path,
) -> Result<Vec<HistoricalPoolStatistics>, Box<dyn std::error::Error>> {
    // Deserialize PoolStatistics from yaml format
    let filename = dirname.join("history.yml");
    let yaml = tokio::fs::read_to_string(filename).await?;
    let statistics = serde_yaml::from_str(&yaml)?;

    Ok(statistics)
}

pub async fn append_history_to_statistics(
    statistics: &PoolStatistics,
    dirname: &Path,
    date: &SystemTime,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut histories = load_history(dirname).await.unwrap_or_else(|_| Vec::new());

    let history = HistoricalPoolStatistics {
        date: date.duration_since(SystemTime::UNIX_EPOCH)?.as_secs(),
        longest_chain: statistics.longest_chain,
        nb_chunk: statistics.nb_chunk,
        nb_ref: statistics.nb_ref,
        size: statistics.size,
        compressed_size: statistics.compressed_size,
        unused_size: statistics.unused_size,
    };

    histories.push(history);

    // Serialize PoolStatistics in yaml format
    let filename = dirname.join("history.yml");
    let yaml = serde_yaml::to_string(&histories)?;
    write(filename, yaml).await?;

    Ok(())
}

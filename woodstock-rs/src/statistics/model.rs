use serde::{Deserialize, Serialize};

/// Represents the statistics of a backup pool.
#[derive(Serialize, Deserialize, Default)]
pub struct PoolStatistics {
    /// The length of the longest chain in the pool.
    #[serde(rename = "longestChain")]
    pub longest_chain: u32,
    /// The number of chunks in the pool.
    #[serde(rename = "nbChunk")]
    pub nb_chunk: u32,
    /// The number of references in the pool.
    #[serde(rename = "nbRef")]
    pub nb_ref: u32,
    /// The total size of the pool in bytes.
    #[serde(rename = "size")]
    pub size: u64,
    /// The total compressed size of the pool in bytes.
    #[serde(rename = "compressedSize")]
    pub compressed_size: u64,
    /// The total unused size in the pool in bytes.
    #[serde(rename = "unusedSize")]
    pub unused_size: u64,
}

/// Represents the historical statistics of a backup pool at a specific date.
#[derive(Serialize, Deserialize, Default)]
pub struct HistoricalPoolStatistics {
    /// The date of the statistics (as a UNIX timestamp).
    #[serde(rename = "date")]
    pub date: u64,
    /// The length of the longest chain in the pool.
    #[serde(rename = "longestChain")]
    pub longest_chain: u32,
    /// The number of chunks in the pool.
    #[serde(rename = "nbChunk")]
    pub nb_chunk: u32,
    /// The number of references in the pool.
    #[serde(rename = "nbRef")]
    pub nb_ref: u32,
    /// The total size of the pool in bytes.
    #[serde(rename = "size")]
    pub size: u64,
    /// The total compressed size of the pool in bytes.
    #[serde(rename = "compressedSize")]
    pub compressed_size: u64,
    /// The total unused size in the pool in bytes.
    #[serde(rename = "unusedSize")]
    pub unused_size: u64,
}

/// Represents the usage statistics of a host in a backup pool, including backup history.
#[derive(Serialize, Deserialize, Default)]
pub struct HostStatsUsage {
    /// The length of the longest chain in the pool.
    #[serde(rename = "longestChain")]
    pub longest_chain: u32,
    /// The number of chunks in the pool.
    #[serde(rename = "nbChunk")]
    pub nb_chunk: u32,
    /// The number of references in the pool.
    #[serde(rename = "nbRef")]
    pub nb_ref: u32,
    /// The total size of the pool in bytes.
    #[serde(rename = "size")]
    pub size: u64,
    /// The total compressed size of the pool in bytes.
    #[serde(rename = "compressedSize")]
    pub compressed_size: u64,
    /// The total unused size in the pool in bytes.
    #[serde(rename = "unusedSize")]
    pub unused_size: u64,
    /// The number of backups performed.
    #[serde(rename = "backupCount")]
    backup_count: u32,
    /// The size of the last backup in bytes.
    #[serde(rename = "lastBackupSize")]
    last_backup_size: u64,
    /// The time of the last backup (as a UNIX timestamp).
    #[serde(rename = "lastBackupTime")]
    last_backup_time: u64,
    /// The age of the last backup in seconds.
    #[serde(rename = "lastBackupAge")]
    last_backup_age: u64,
    /// The duration of the last backup in seconds.
    #[serde(rename = "lastBackupDuration")]
    last_backup_duration: u64,
    /// Whether the last backup was complete (1) or not (0).
    #[serde(rename = "lastBackupComplete")]
    last_backup_complete: u32,
}

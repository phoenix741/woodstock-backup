use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

pub trait WithDate {
    fn date(&self) -> DateTime<Local>;
}

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
    #[serde(
        rename = "size",
        deserialize_with = "crate::utils::serde::deserialize_u64_or_string"
    )]
    pub size: u64,
    /// The total compressed size of the pool in bytes.
    #[serde(
        rename = "compressedSize",
        deserialize_with = "crate::utils::serde::deserialize_u64_or_string"
    )]
    pub compressed_size: u64,
    /// The total unused size in the pool in bytes.
    #[serde(
        rename = "unusedSize",
        deserialize_with = "crate::utils::serde::deserialize_u64_or_string"
    )]
    pub unused_size: u64,
}

/// Represents the historical statistics of a backup pool at a specific date.
#[derive(Serialize, Deserialize, Eq, PartialEq, Default, Debug)]
pub struct HistoricalPoolStatistics {
    /// The date of the statistics (as a UNIX timestamp).
    #[serde(
        rename = "date",
        deserialize_with = "crate::utils::serde::deserialize_local_datetime"
    )]
    pub date: DateTime<Local>,
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
    #[serde(
        rename = "size",
        deserialize_with = "crate::utils::serde::deserialize_u64_or_string"
    )]
    pub size: u64,
    /// The total compressed size of the pool in bytes.
    #[serde(
        rename = "compressedSize",
        deserialize_with = "crate::utils::serde::deserialize_u64_or_string"
    )]
    pub compressed_size: u64,
    /// The total unused size in the pool in bytes.
    #[serde(
        rename = "unusedSize",
        deserialize_with = "crate::utils::serde::deserialize_u64_or_string"
    )]
    pub unused_size: u64,
}

impl WithDate for HistoricalPoolStatistics {
    fn date(&self) -> DateTime<Local> {
        self.date
    }
}

/// Instantané d'usage disque (aligné sur StatsDiskUsage TS)
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct StatsDiskUsage {
    pub fstype: String,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_u64_or_string")]
    pub size: u64,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_u64_or_string")]
    pub used: u64,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_u64_or_string")]
    pub free: u64,
}

/// Entrée historisée d'usage disque (StatsDiskUsage + date ms)
#[derive(Serialize, Deserialize, Eq, PartialEq, Default, Clone)]
pub struct HistoricalDiskStatistics {
    pub fstype: String,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_u64_or_string")]
    pub size: u64,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_u64_or_string")]
    pub used: u64,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_u64_or_string")]
    pub free: u64,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_local_datetime")]
    pub date: DateTime<Local>,
}

impl WithDate for HistoricalDiskStatistics {
    fn date(&self) -> DateTime<Local> {
        self.date
    }
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
    #[serde(
        rename = "size",
        deserialize_with = "crate::utils::serde::deserialize_u64_or_string"
    )]
    pub size: u64,
    /// The total compressed size of the pool in bytes.
    #[serde(
        rename = "compressedSize",
        deserialize_with = "crate::utils::serde::deserialize_u64_or_string"
    )]
    pub compressed_size: u64,
    /// The total unused size in the pool in bytes.
    #[serde(
        rename = "unusedSize",
        deserialize_with = "crate::utils::serde::deserialize_u64_or_string"
    )]
    pub unused_size: u64,
    /// The number of backups performed.
    #[serde(rename = "backupCount")]
    pub backup_count: u32,
    /// The size of the last backup in bytes.
    #[serde(
        rename = "lastBackupSize",
        deserialize_with = "crate::utils::serde::deserialize_u64_or_string"
    )]
    pub last_backup_size: u64,
    /// The time of the last backup (as a UNIX timestamp).
    #[serde(rename = "lastBackupTime")]
    pub last_backup_time: u64,
    /// The age of the last backup in seconds.
    #[serde(rename = "lastBackupAge")]
    pub last_backup_age: u64,
    /// The duration of the last backup in seconds.
    #[serde(rename = "lastBackupDuration")]
    pub last_backup_duration: u64,
    /// Whether the last backup was complete (1) or not (0).
    #[serde(rename = "lastBackupComplete")]
    pub last_backup_complete: u32,
}

impl HostStatsUsage {
    pub fn new(
        longest_chain: u32,
        nb_chunk: u32,
        nb_ref: u32,
        size: u64,
        compressed_size: u64,
        unused_size: u64,
    ) -> Self {
        Self {
            longest_chain,
            nb_chunk,
            nb_ref,
            size,
            compressed_size,
            unused_size,
            backup_count: 0,
            last_backup_size: 0,
            last_backup_time: 0,
            last_backup_age: 0,
            last_backup_duration: 0,
            last_backup_complete: 0,
        }
    }
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct PoolStatistics {
    #[serde(rename = "longestChain")]
    pub longest_chain: u32,
    #[serde(rename = "nbChunk")]
    pub nb_chunk: u32,
    #[serde(rename = "nbRef")]
    pub nb_ref: u32,
    #[serde(rename = "size")]
    pub size: u64,
    #[serde(rename = "compressedSize")]
    pub compressed_size: u64,
    #[serde(rename = "unusedSize")]
    pub unused_size: u64,
}

#[derive(Serialize, Deserialize, Default)]
pub struct HistoricalPoolStatistics {
    #[serde(rename = "date")]
    pub date: u64,
    #[serde(rename = "longestChain")]
    pub longest_chain: u32,
    #[serde(rename = "nbChunk")]
    pub nb_chunk: u32,
    #[serde(rename = "nbRef")]
    pub nb_ref: u32,
    #[serde(rename = "size")]
    pub size: u64,
    #[serde(rename = "compressedSize")]
    pub compressed_size: u64,
    #[serde(rename = "unusedSize")]
    pub unused_size: u64,
}

#[derive(Serialize, Deserialize, Default)]
pub struct HostStatsUsage {
    #[serde(rename = "longestChain")]
    pub longest_chain: u32,
    #[serde(rename = "nbChunk")]
    pub nb_chunk: u32,
    #[serde(rename = "nbRef")]
    pub nb_ref: u32,
    #[serde(rename = "size")]
    pub size: u64,
    #[serde(rename = "compressedSize")]
    pub compressed_size: u64,
    #[serde(rename = "unusedSize")]
    pub unused_size: u64,

    #[serde(rename = "backupCount")]
    backup_count: u32,
    #[serde(rename = "lastBackupSize")]
    last_backup_size: u64,
    #[serde(rename = "lastBackupTime")]
    last_backup_time: u64,
    #[serde(rename = "lastBackupAge")]
    last_backup_age: u64,
    #[serde(rename = "lastBackupDuration")]
    last_backup_duration: u64,
    #[serde(rename = "lastBackupComplete")]
    last_backup_complete: u32,
}

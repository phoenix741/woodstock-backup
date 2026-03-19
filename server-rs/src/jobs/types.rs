//! Définition des payloads de jobs, en parité avec la version TypeScript (BullMQ)

use std::net::SocketAddr;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use woodstock::config::HostConfiguration;

/// Nom des files (queues) identiques à l'implémentation TS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueName {
    Schedule,
    Backup,
    Interactive,
    Maintenance,
}

impl QueueName {
    pub fn as_str(&self) -> &'static str {
        match self {
            QueueName::Schedule => "schedule",
            QueueName::Backup => "backup",
            QueueName::Interactive => "interactive",
            QueueName::Maintenance => "maintenance",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupJobData {
    pub host: String,
    pub config: HostConfiguration,
    /// UUID v7 for this backup (primary key / directory name)
    pub id: Uuid,
    /// Sequential number for display only
    pub number: usize,
    /// UUID v7 of the previous backup (for resume detection)
    pub previous_id: Option<Uuid>,
    pub ip: Option<SocketAddr>,
    pub start_date: Option<DateTime<Local>>,
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JobRestoreDataSelection {
    pub share: String,
    pub selection: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RestoreJobData {
    pub host: String,
    pub config: Option<HostConfiguration>,
    /// UUID v7 of the backup to restore
    pub id: Uuid,
    /// Sequential number for display only
    pub number: usize,
    pub ip: Option<SocketAddr>,
    pub start_date: Option<DateTime<Local>>,

    // Destination directory and file to restore
    pub destination_directory: String,
    pub files: Vec<JobRestoreDataSelection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoveJobData {
    pub host: String,
    pub config: Option<HostConfiguration>,
    /// UUID v7 of the backup to remove
    pub id: Uuid,
    /// Sequential number for display only
    pub number: usize,
    pub start_date: Option<DateTime<Local>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatsJobData {}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CleanupPoolJobData {
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FsckJobData {
    pub dry_run: bool,
    pub verify_chunks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum MaintenanceJobData {
    CleanupPool(CleanupPoolJobData),
    Fsck(FsckJobData),
    Stats(StatsJobData),
}

impl Default for MaintenanceJobData {
    fn default() -> Self {
        MaintenanceJobData::CleanupPool(CleanupPoolJobData { target: None })
    }
}

/// Génère la clé d'unicité identique à TS pour les backups
pub fn backup_unique_key(host: &str) -> String {
    format!("backup::{}", host)
}

/// Jobs de la file Backup: soit un backup (Save), soit une suppression (Remove)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum BackupQueueJob {
    Save(BackupJobData),
    Remove(RemoveJobData),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ScheduleQueueJob {
    #[default]
    Backup,
}

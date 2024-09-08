use serde::{Deserialize, Serialize};

// ************ Schedule ************

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledBackupToKeep {
    pub hourly: Option<u8>,
    pub daily: Option<u8>,
    pub weekly: Option<u8>,
    pub monthly: Option<u8>,
    pub yearly: Option<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub activated: Option<bool>,
    pub backup_period: Option<u8>,
    pub backup_to_keep: Option<ScheduledBackupToKeep>,
}

// ************* Host **************

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackupTaskShare {
    pub name: String,
    pub includes: Option<Vec<String>>,
    pub excludes: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecuteCommandOperation {
    pub command: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackupOperation {
    pub shares: Vec<BackupTaskShare>,
    pub includes: Option<Vec<String>>,
    pub excludes: Option<Vec<String>>,
    pub timeout: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostConfigOperation {
    pub pre_commands: Option<Vec<ExecuteCommandOperation>>,
    pub operation: Option<BackupOperation>,
    pub post_commands: Option<Vec<ExecuteCommandOperation>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostConfiguration {
    pub is_local: Option<bool>,
    pub password: String,
    pub addresses: Option<Vec<String>>,
    pub operations: HostConfigOperation,
    pub schedule: Option<Schedule>,
}

// ************ Backup ****************

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Backup {
    pub number: usize,
    pub completed: bool,

    pub start_date: u64,
    pub end_date: Option<u64>,

    #[serde(default)]
    pub error_count: usize,

    pub file_count: usize,
    pub new_file_count: usize,
    pub removed_file_count: usize,
    pub modified_file_count: usize,
    pub existing_file_count: usize,

    pub file_size: u64,
    pub new_file_size: u64,
    pub modified_file_size: u64,
    pub existing_file_size: u64,

    pub compressed_file_size: u64,
    pub new_compressed_file_size: u64,
    pub modified_compressed_file_size: u64,
    pub existing_compressed_file_size: u64,

    pub speed: f64,
}

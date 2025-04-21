use serde::{Deserialize, Serialize};
use std::fmt;

use super::DEFAULT_PORT;

/// # Configuration Data Models
///
/// This module defines the main data structures used for configuration in the Woodstock backup system.
/// It includes models for backup scheduling, host configuration, backup operations, and backup status.
///
/// ## Main Structures
///
/// - [`ScheduledBackupToKeep`]: Defines retention policy for backups (hourly, daily, etc.).
/// - [`Schedule`]: Defines backup scheduling and retention.
/// - [`BackupTaskShare`]: Represents a share (directory or volume) to be backed up.
/// - [`ExecuteCommandOperation`]: Represents a shell command to execute before/after backup.
/// - [`BackupOperation`]: Describes a backup operation (shares, includes, excludes, timeout).
/// - [`HostConfigOperation`]: Describes pre/post commands and backup operation for a host.
/// - [`HostConfiguration`]: Full configuration for a backup host.
/// - [`BackupStatus`]: Enum for backup state (in progress, completed, failed, etc.).
/// - [`Backup`]: Metadata for a single backup run.
///
/// ## Usage
///
/// These structures are typically (de)serialized from YAML configuration files and used throughout
/// the backup system for scheduling, execution, and reporting.
///
/// ## Error Handling & Panics
///
/// - All structures are designed for safe (de)serialization and do not panic.
/// - Enum methods return booleans and do not panic.

// ************ Schedule ************

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Defines retention policy for backups (how many to keep for each period).
pub struct ScheduledBackupToKeep {
    pub hourly: Option<u8>,
    pub daily: Option<u8>,
    pub weekly: Option<u8>,
    pub monthly: Option<u8>,
    pub yearly: Option<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Defines backup scheduling and retention policy for a host.
pub struct Schedule {
    pub activated: Option<bool>,
    pub backup_period: Option<u8>,
    pub backup_to_keep: Option<ScheduledBackupToKeep>,
}

// ************* Host **************

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Represents a share (directory or volume) to be backed up.
pub struct BackupTaskShare {
    pub name: String,
    pub includes: Option<Vec<String>>,
    pub excludes: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
/// Represents a shell command to execute before or after backup.
pub struct ExecuteCommandOperation {
    pub command: String,
}

impl fmt::Display for ExecuteCommandOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.command)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Describes a backup operation (shares, includes, excludes, timeout).
pub struct BackupOperation {
    pub shares: Vec<BackupTaskShare>,
    pub includes: Option<Vec<String>>,
    pub excludes: Option<Vec<String>>,
    pub timeout: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Describes pre/post commands and backup operation for a host.
pub struct HostConfigOperation {
    pub pre_commands: Option<Vec<ExecuteCommandOperation>>,
    pub operation: Option<BackupOperation>,
    pub post_commands: Option<Vec<ExecuteCommandOperation>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Full configuration for a backup host.
pub struct HostConfiguration {
    pub password: String,
    pub addresses: Option<Vec<String>>,
    #[serde(default = "default_port")]
    pub port: u16,
    pub operations: HostConfigOperation,
    pub schedule: Option<Schedule>,
}

fn default_port() -> u16 {
    DEFAULT_PORT
}

// ************ Backup ****************

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// Enum for backup state (in progress, completed, failed, etc.).
pub enum BackupStatus {
    InProgress,
    Finishing,
    Completed,
    Aborted,
    Failed,
}

impl BackupStatus {
    /// Returns true if the backup is finished (completed, aborted, or failed).
    #[must_use]
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            BackupStatus::Completed | BackupStatus::Aborted | BackupStatus::Failed
        )
    }

    /// Returns true if the backup was aborted or failed.
    #[must_use]
    pub fn is_aborted(&self) -> bool {
        matches!(self, BackupStatus::Aborted | BackupStatus::Failed)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Metadata for a single backup run.
pub struct Backup {
    pub number: usize,
    pub status: BackupStatus,

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

    pub agent_version: Option<String>,
}

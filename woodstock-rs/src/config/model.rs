use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

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
    pub hourly: Option<usize>,
    pub daily: Option<usize>,
    pub weekly: Option<usize>,
    pub monthly: Option<usize>,
    pub yearly: Option<usize>,
    /// Maximum number of yearly representatives to keep (oldest dropped first).
    /// `None` means unlimited — all years are kept.
    #[serde(default)]
    pub yearly_limit: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Defines backup scheduling and retention policy for a host.
pub struct Schedule {
    pub activated: Option<bool>,
    pub backup_period: Option<i64>,
    pub backup_to_keep: Option<ScheduledBackupToKeep>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationScheduler {
    pub wakeup_schedule: String,
    pub nightly_schedule: String,
    pub default_schedule: Schedule,
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
/// Represents the stage within the Finishing phase
pub enum FinishingStatus {
    ToCompact,
    ToCountRef,
    ToAddInPool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// Represents which operation failed
pub enum FailedStatus {
    Compact,
    RefCount,
    InPool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// Represents the stage within the Removing phase
pub enum RemovingStatus {
    ToRemoveInPool,
    RemoveFromHost,
    ToRemove,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", content = "details")]
/// Enum for backup state (in progress, completed, failed, etc.).
/// Some variants contain additional context about the current stage or failure point.
pub enum BackupStatus {
    InProgress,
    Finishing(FinishingStatus),
    Completed,
    Aborting(FinishingStatus),
    Aborted,
    /// Terminal state for a backup deliberately stopped by the user (as
    /// opposed to `Aborted`, which covers critical errors and lock loss).
    /// Winds down through the same `Aborting(stage)` finalization pipeline
    /// as `Aborted` — only the final persisted status differs.
    Cancelled,
    Failed(FailedStatus),
    Removing(RemovingStatus),
}

impl BackupStatus {
    /// Returns true if the backup is finished (completed, aborted, cancelled, or failed).
    #[must_use]
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            BackupStatus::Completed
                | BackupStatus::Aborted
                | BackupStatus::Cancelled
                | BackupStatus::Failed(_)
        )
    }

    /// Returns true if the backup was aborted, cancelled, or is in the process of winding down.
    #[must_use]
    pub fn is_aborted(&self) -> bool {
        matches!(
            self,
            BackupStatus::Aborting(_)
                | BackupStatus::Aborted
                | BackupStatus::Cancelled
                | BackupStatus::Failed(_)
        )
    }

    /// Returns true if the backup can be resumed (not finished and not in a removal state that's complete).
    /// Used to detect if a backup operation should be resumed after a crash.
    #[must_use]
    pub fn is_resumable(&self) -> bool {
        !self.is_finished()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Metadata for a single backup run.
pub struct Backup {
    /// Unique temporal identifier (UUID v7). Primary key used for filesystem routing.
    pub id: Uuid,
    /// Sequential display number. Kept for human-readable display in the front-end only.
    pub number: usize,
    pub status: BackupStatus,

    #[serde(deserialize_with = "crate::utils::serde::deserialize_local_datetime")]
    pub start_date: DateTime<Local>,
    #[serde(
        default,
        deserialize_with = "crate::utils::serde::deserialize_option_local_datetime"
    )]
    pub end_date: Option<DateTime<Local>>,

    #[serde(default)]
    pub error_count: usize,

    #[serde(default)]
    pub error_message: Option<String>,

    pub file_count: usize,
    pub new_file_count: usize,
    pub removed_file_count: usize,
    pub modified_file_count: usize,
    pub existing_file_count: usize,

    #[serde(deserialize_with = "crate::utils::serde::deserialize_u64_or_string")]
    pub file_size: u64,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_u64_or_string")]
    pub new_file_size: u64,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_u64_or_string")]
    pub modified_file_size: u64,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_u64_or_string")]
    pub existing_file_size: u64,

    #[serde(deserialize_with = "crate::utils::serde::deserialize_u64_or_string")]
    pub compressed_file_size: u64,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_u64_or_string")]
    pub new_compressed_file_size: u64,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_u64_or_string")]
    pub modified_compressed_file_size: u64,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_u64_or_string")]
    pub existing_compressed_file_size: u64,

    pub speed: f64,

    pub agent_version: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_resumable_for_backup_states() {
        // Resumable states (all non-finished states)
        assert!(BackupStatus::InProgress.is_resumable());
        assert!(BackupStatus::Finishing(FinishingStatus::ToCompact).is_resumable());
        assert!(BackupStatus::Finishing(FinishingStatus::ToCountRef).is_resumable());
        assert!(BackupStatus::Finishing(FinishingStatus::ToAddInPool).is_resumable());
        assert!(BackupStatus::Aborting(FinishingStatus::ToCompact).is_resumable());
        assert!(BackupStatus::Aborting(FinishingStatus::ToCountRef).is_resumable());
        assert!(BackupStatus::Aborting(FinishingStatus::ToAddInPool).is_resumable());

        // Resumable removal states
        assert!(BackupStatus::Removing(RemovingStatus::ToRemoveInPool).is_resumable());
        assert!(BackupStatus::Removing(RemovingStatus::RemoveFromHost).is_resumable());
    }

    #[test]
    fn test_is_not_resumable_for_finished_states() {
        // Non-resumable states (all finished states)
        assert!(!BackupStatus::Completed.is_resumable());
        assert!(!BackupStatus::Aborted.is_resumable());
        assert!(!BackupStatus::Cancelled.is_resumable());
        assert!(!BackupStatus::Failed(FailedStatus::Compact).is_resumable());
        assert!(!BackupStatus::Failed(FailedStatus::RefCount).is_resumable());
        assert!(!BackupStatus::Failed(FailedStatus::InPool).is_resumable());
    }

    #[test]
    fn test_is_finished() {
        assert!(!BackupStatus::InProgress.is_finished());
        assert!(!BackupStatus::Finishing(FinishingStatus::ToCompact).is_finished());
        assert!(!BackupStatus::Finishing(FinishingStatus::ToCountRef).is_finished());
        assert!(!BackupStatus::Finishing(FinishingStatus::ToAddInPool).is_finished());
        assert!(!BackupStatus::Aborting(FinishingStatus::ToCompact).is_finished());
        assert!(!BackupStatus::Aborting(FinishingStatus::ToCountRef).is_finished());
        assert!(!BackupStatus::Aborting(FinishingStatus::ToAddInPool).is_finished());
        assert!(BackupStatus::Completed.is_finished());
        assert!(BackupStatus::Aborted.is_finished());
        assert!(BackupStatus::Cancelled.is_finished());
        assert!(BackupStatus::Failed(FailedStatus::Compact).is_finished());
        assert!(BackupStatus::Failed(FailedStatus::RefCount).is_finished());
        assert!(BackupStatus::Failed(FailedStatus::InPool).is_finished());
        assert!(!BackupStatus::Removing(RemovingStatus::ToRemoveInPool).is_finished());
        assert!(!BackupStatus::Removing(RemovingStatus::RemoveFromHost).is_finished());
    }

    #[test]
    fn test_is_aborted() {
        assert!(!BackupStatus::InProgress.is_aborted());
        assert!(!BackupStatus::Finishing(FinishingStatus::ToCompact).is_aborted());
        assert!(!BackupStatus::Finishing(FinishingStatus::ToCountRef).is_aborted());
        assert!(!BackupStatus::Finishing(FinishingStatus::ToAddInPool).is_aborted());
        assert!(BackupStatus::Aborting(FinishingStatus::ToCompact).is_aborted());
        assert!(BackupStatus::Aborting(FinishingStatus::ToCountRef).is_aborted());
        assert!(BackupStatus::Aborting(FinishingStatus::ToAddInPool).is_aborted());
        assert!(!BackupStatus::Completed.is_aborted());
        assert!(BackupStatus::Aborted.is_aborted());
        assert!(BackupStatus::Cancelled.is_aborted());
        assert!(BackupStatus::Failed(FailedStatus::Compact).is_aborted());
        assert!(BackupStatus::Failed(FailedStatus::RefCount).is_aborted());
        assert!(BackupStatus::Failed(FailedStatus::InPool).is_aborted());
        assert!(!BackupStatus::Removing(RemovingStatus::ToRemoveInPool).is_aborted());
        assert!(!BackupStatus::Removing(RemovingStatus::RemoveFromHost).is_aborted());
    }
}

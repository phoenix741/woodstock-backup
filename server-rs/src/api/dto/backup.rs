use async_graphql::SimpleObject;
use chrono::Local;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use woodstock::config::{
    Backup as WoodstockBackup, BackupStatus, FailedStatus, FinishingStatus, RemovingStatus,
    ShareRecord,
};

use crate::graphql::scalars::BigIntScalar;

#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, ToSchema, async_graphql::Enum,
)]
#[serde(rename_all = "camelCase")]
pub enum BackupStatusTypeDto {
    InProgress,
    Finishing,
    Completed,
    Aborting,
    Aborted,
    Failed,
    Removing,
}

#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, ToSchema, async_graphql::Enum,
)]
#[serde(rename_all = "camelCase")]
pub enum FinishingStageDto {
    ToCompact,
    ToCountRef,
    ToAddInPool,
}

#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, ToSchema, async_graphql::Enum,
)]
#[serde(rename_all = "camelCase")]
pub enum AbortingStageDto {
    ToCompact,
    ToCountRef,
    ToAddInPool,
}

#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, ToSchema, async_graphql::Enum,
)]
#[serde(rename_all = "camelCase")]
pub enum FailedStageDto {
    Compact,
    RefCount,
    InPool,
}

#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, ToSchema, async_graphql::Enum,
)]
#[serde(rename_all = "camelCase")]
pub enum RemovingStageDto {
    ToRemoveInPool,
    RemoveFromHost,
    ToRemove,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct BackupStatusDto {
    pub status_type: BackupStatusTypeDto,
    pub finishing_stage: Option<FinishingStageDto>,
    pub aborting_stage: Option<AbortingStageDto>,
    pub failed_stage: Option<FailedStageDto>,
    pub removing_stage: Option<RemovingStageDto>,
}

impl From<BackupStatus> for BackupStatusDto {
    fn from(status: BackupStatus) -> Self {
        match status {
            BackupStatus::InProgress => BackupStatusDto {
                status_type: BackupStatusTypeDto::InProgress,
                finishing_stage: None,
                aborting_stage: None,
                failed_stage: None,
                removing_stage: None,
            },
            BackupStatus::Finishing(stage) => {
                let finishing_stage = match stage {
                    FinishingStatus::ToCompact => FinishingStageDto::ToCompact,
                    FinishingStatus::ToCountRef => FinishingStageDto::ToCountRef,
                    FinishingStatus::ToAddInPool => FinishingStageDto::ToAddInPool,
                };
                BackupStatusDto {
                    status_type: BackupStatusTypeDto::Finishing,
                    finishing_stage: Some(finishing_stage),
                    aborting_stage: None,
                    failed_stage: None,
                    removing_stage: None,
                }
            }
            BackupStatus::Completed => BackupStatusDto {
                status_type: BackupStatusTypeDto::Completed,
                finishing_stage: None,
                aborting_stage: None,
                failed_stage: None,
                removing_stage: None,
            },
            BackupStatus::Aborting(stage) => {
                let aborting_stage = match stage {
                    FinishingStatus::ToCompact => AbortingStageDto::ToCompact,
                    FinishingStatus::ToCountRef => AbortingStageDto::ToCountRef,
                    FinishingStatus::ToAddInPool => AbortingStageDto::ToAddInPool,
                };
                BackupStatusDto {
                    status_type: BackupStatusTypeDto::Aborting,
                    finishing_stage: None,
                    aborting_stage: Some(aborting_stage),
                    failed_stage: None,
                    removing_stage: None,
                }
            }
            BackupStatus::Aborted => BackupStatusDto {
                status_type: BackupStatusTypeDto::Aborted,
                finishing_stage: None,
                aborting_stage: None,
                failed_stage: None,
                removing_stage: None,
            },
            BackupStatus::Failed(stage) => {
                let failed_stage = match stage {
                    FailedStatus::Compact => FailedStageDto::Compact,
                    FailedStatus::RefCount => FailedStageDto::RefCount,
                    FailedStatus::InPool => FailedStageDto::InPool,
                };
                BackupStatusDto {
                    status_type: BackupStatusTypeDto::Failed,
                    finishing_stage: None,
                    aborting_stage: None,
                    failed_stage: Some(failed_stage),
                    removing_stage: None,
                }
            }
            BackupStatus::Removing(stage) => {
                let removing_stage = match stage {
                    RemovingStatus::ToRemoveInPool => RemovingStageDto::ToRemoveInPool,
                    RemovingStatus::RemoveFromHost => RemovingStageDto::RemoveFromHost,
                    RemovingStatus::ToRemove => RemovingStageDto::ToRemove,
                };
                BackupStatusDto {
                    status_type: BackupStatusTypeDto::Removing,
                    finishing_stage: None,
                    aborting_stage: None,
                    failed_stage: None,
                    removing_stage: Some(removing_stage),
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Backup {
    pub id: String,
    pub number: usize,
    pub status: BackupStatusDto,
    pub error_count: usize,
    pub error_message: Option<String>,
    pub start_date: chrono::DateTime<Local>,
    pub end_date: Option<chrono::DateTime<Local>>,
    pub file_count: usize,
    pub new_file_count: usize,
    pub existing_file_count: usize,
    pub removed_file_count: usize,
    pub modified_file_count: usize,

    pub file_size: u64,
    pub existing_file_size: u64,
    pub new_file_size: u64,
    pub modified_file_size: u64,
    pub compressed_file_size: u64,
    pub existing_compressed_file_size: u64,
    pub new_compressed_file_size: u64,
    pub modified_compressed_file_size: u64,

    pub speed: f64,
    pub agent_version: Option<String>,
}

/// Retention category DTO — mirrors [`woodstock::server::backup::retention::RetentionCategory`].
#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, ToSchema, async_graphql::Enum,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RetentionCategoryDto {
    /// Representative of an hourly slot.
    Hourly,
    /// Representative of a daily slot.
    Daily,
    /// Representative of a weekly (ISO-week) slot.
    Weekly,
    /// Representative of a monthly slot.
    Monthly,
    /// Representative of a yearly slot.
    Yearly,
    /// Not retained by any slot — scheduled for deletion.
    Surplus,
    /// Most recent terminal backup — protected from deletion.
    LastBackup,
}

impl From<woodstock::server::backup::retention::RetentionCategory> for RetentionCategoryDto {
    fn from(cat: woodstock::server::backup::retention::RetentionCategory) -> Self {
        use woodstock::server::backup::retention::RetentionCategory as C;
        match cat {
            C::Hourly => Self::Hourly,
            C::Daily => Self::Daily,
            C::Weekly => Self::Weekly,
            C::Monthly => Self::Monthly,
            C::Yearly => Self::Yearly,
            C::Surplus => Self::Surplus,
            C::LastBackup => Self::LastBackup,
        }
    }
}

impl From<WoodstockBackup> for Backup {
    fn from(backup: WoodstockBackup) -> Self {
        Self {
            id: backup.id.to_string(),
            number: backup.number,
            status: BackupStatusDto::from(backup.status),
            error_count: backup.error_count,
            error_message: backup.error_message,
            start_date: backup.start_date,
            end_date: backup.end_date,
            file_count: backup.file_count,
            new_file_count: backup.new_file_count,
            existing_file_count: backup.existing_file_count,
            removed_file_count: backup.removed_file_count,
            modified_file_count: backup.modified_file_count,
            file_size: backup.file_size,
            existing_file_size: backup.existing_file_size,
            new_file_size: backup.new_file_size,
            modified_file_size: backup.modified_file_size,
            compressed_file_size: backup.compressed_file_size,
            existing_compressed_file_size: backup.existing_compressed_file_size,
            new_compressed_file_size: backup.new_compressed_file_size,
            modified_compressed_file_size: backup.modified_compressed_file_size,
            speed: backup.speed,
            agent_version: backup.agent_version,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct FileListProgression {
    pub file_size: BigIntScalar,
    pub new_file_size: BigIntScalar,
    pub modified_file_size: BigIntScalar,

    pub new_file_count: usize,
    pub modified_file_count: usize,
    pub removed_file_count: usize,
}

impl From<woodstock::server::progression::FileListProgression> for FileListProgression {
    fn from(p: woodstock::server::progression::FileListProgression) -> Self {
        Self {
            file_size: BigIntScalar(p.file_size),
            new_file_size: BigIntScalar(p.new_file_size),
            modified_file_size: BigIntScalar(p.modified_file_size),
            new_file_count: p.new_file_count,
            modified_file_count: p.modified_file_count,
            removed_file_count: p.removed_file_count,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct BackupProgression {
    pub start_date: chrono::DateTime<Local>,
    pub start_transfer_date: Option<chrono::DateTime<Local>>,
    pub end_transfer_date: Option<chrono::DateTime<Local>>,
    pub file_size: BigIntScalar,
    pub new_file_size: BigIntScalar,
    pub modified_file_size: BigIntScalar,
    pub compressed_file_size: BigIntScalar,
    pub new_compressed_file_size: BigIntScalar,
    pub modified_compressed_file_size: BigIntScalar,
    pub file_count: usize,
    pub new_file_count: usize,
    pub modified_file_count: usize,
    pub removed_file_count: usize,
    pub error_count: usize,
    pub speed: f64,
    pub percent: f64,
    pub progress_current: BigIntScalar,
    pub progress_max: BigIntScalar,
}

impl From<woodstock::server::progression::BackupProgression> for BackupProgression {
    fn from(p: woodstock::server::progression::BackupProgression) -> Self {
        let speed = p.speed();
        let percent = p.percent();

        Self {
            start_date: p.start_date,
            start_transfer_date: p.start_transfer_date,
            end_transfer_date: p.end_transfer_date,
            file_size: BigIntScalar(p.file_size),
            new_file_size: BigIntScalar(p.new_file_size),
            modified_file_size: BigIntScalar(p.modified_file_size),
            compressed_file_size: BigIntScalar(p.compressed_file_size),
            new_compressed_file_size: BigIntScalar(p.new_compressed_file_size),
            modified_compressed_file_size: BigIntScalar(p.modified_compressed_file_size),
            file_count: p.file_count,
            new_file_count: p.new_file_count,
            modified_file_count: p.modified_file_count,
            removed_file_count: p.removed_file_count,
            error_count: p.error_count,
            speed,
            percent,
            progress_current: BigIntScalar(p.progress_current),
            progress_max: BigIntScalar(p.progress_max),
        }
    }
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq)]
pub enum ExecuteCommandExecutionState {
    Waiting,
    InProgress,
    Success,
    Failed,
}

impl From<woodstock::server::backup::save_state::ExecuteCommandExecutionState>
    for ExecuteCommandExecutionState
{
    fn from(s: woodstock::server::backup::save_state::ExecuteCommandExecutionState) -> Self {
        use woodstock::server::backup::save_state::ExecuteCommandExecutionState as Src;
        match s {
            Src::Waiting => ExecuteCommandExecutionState::Waiting,
            Src::InProgress => ExecuteCommandExecutionState::InProgress,
            Src::Success(_) => ExecuteCommandExecutionState::Success,
            Src::Failed(_) => ExecuteCommandExecutionState::Failed,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct ExecuteCommandState {
    pub command: crate::api::dto::ExecuteCommandOperation,
    pub execution_state: ExecuteCommandExecutionState,
}

impl From<woodstock::server::backup::save_state::ExecuteCommandState> for ExecuteCommandState {
    fn from(s: woodstock::server::backup::save_state::ExecuteCommandState) -> Self {
        Self {
            command: s.command.into(),
            execution_state: s.execution_state.into(),
        }
    }
}

#[derive(
    async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize, ToSchema,
)]
pub enum SnapshotMethodDto {
    None,
    Btrfs,
    Vss,
}

impl From<woodstock::config::ShareSnapshotMethod> for SnapshotMethodDto {
    fn from(m: woodstock::config::ShareSnapshotMethod) -> Self {
        use woodstock::config::ShareSnapshotMethod as Src;
        match m {
            Src::None => SnapshotMethodDto::None,
            Src::Btrfs => SnapshotMethodDto::Btrfs,
            Src::Vss => SnapshotMethodDto::Vss,
        }
    }
}

/// A share record for a completed backup — path + snapshot method used.
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BackupShareRecord {
    pub path: String,
    pub snapshot_method: SnapshotMethodDto,
    pub snapshot_failure_reason: Option<String>,
}

impl From<ShareRecord> for BackupShareRecord {
    fn from(r: ShareRecord) -> Self {
        Self {
            path: r.path,
            snapshot_method: r.snapshot_method.into(),
            snapshot_failure_reason: r.snapshot_failure_reason,
        }
    }
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq)]
pub enum ShareExecutionState {
    Waiting,
    FileList,
    InProgress,
    Success,
    Failed,
}

impl From<woodstock::server::backup::save_state::ShareExecutionState> for ShareExecutionState {
    fn from(s: woodstock::server::backup::save_state::ShareExecutionState) -> Self {
        use woodstock::server::backup::save_state::ShareExecutionState as Src;
        match s {
            Src::Waiting => ShareExecutionState::Waiting,
            Src::FileList => ShareExecutionState::FileList,
            Src::InProgress => ShareExecutionState::InProgress,
            Src::Success => ShareExecutionState::Success,
            Src::Failed(_) => ShareExecutionState::Failed,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct ShareState {
    pub share: String,
    pub file_list_progression: FileListProgression,
    pub backup_progression: BackupProgression,
    pub execution_state: ShareExecutionState,
    pub snapshot_method: SnapshotMethodDto,
    pub snapshot_failure_reason: Option<String>,
}

impl From<woodstock::server::backup::save_state::ShareState> for ShareState {
    fn from(s: woodstock::server::backup::save_state::ShareState) -> Self {
        Self {
            share: s.share,
            file_list_progression: s.file_list_progression.into(),
            backup_progression: s.backup_progression.into(),
            execution_state: s.execution_state.into(),
            snapshot_method: s.snapshot_method.into(),
            snapshot_failure_reason: s.snapshot_failure_reason,
        }
    }
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq)]
pub enum BackupErrorState {
    AuthenticationError,
    InitializationError,
    CommandExecutionError,
    BackupError,
    CompactError,
    CountReferencesError,
    AddReferencesToPoolError,
    Unknown,
}

impl From<woodstock::server::backup::save_state::ErrorState> for BackupErrorState {
    fn from(s: woodstock::server::backup::save_state::ErrorState) -> Self {
        use woodstock::server::backup::save_state::ErrorState as Src;
        match s {
            Src::AuthenticationError(_) => BackupErrorState::AuthenticationError,
            Src::InitializationError(_) => BackupErrorState::InitializationError,
            Src::CommandExecutionError(_) => BackupErrorState::CommandExecutionError,
            Src::BackupError(_) => BackupErrorState::BackupError,
            Src::CompactError(_) => BackupErrorState::CompactError,
            Src::CountReferencesError(_) => BackupErrorState::CountReferencesError,
            Src::AddReferencesToPoolError(_) => BackupErrorState::AddReferencesToPoolError,
            Src::Unknown(_) => BackupErrorState::Unknown,
        }
    }
}

fn backup_error_state_message(s: &woodstock::server::backup::save_state::ErrorState) -> &String {
    use woodstock::server::backup::save_state::ErrorState as Src;
    match s {
        Src::AuthenticationError(e) => e,
        Src::InitializationError(e) => e,
        Src::CommandExecutionError(e) => e,
        Src::BackupError(e) => e,
        Src::CompactError(e) => e,
        Src::CountReferencesError(e) => e,
        Src::AddReferencesToPoolError(e) => e,
        Src::Unknown(e) => e,
    }
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq)]
pub enum BackupExecutionState {
    Waiting,
    Skipped,
    Authenticate,
    Initialization,
    PreCommands,
    DownloadFileList,
    DownloadChunks,
    PostCommands,
    Compact,
    CountReferences,
    AddReferencesToPool,
    Completed,
}

impl From<woodstock::server::backup::save_state::BackupExecutionState> for BackupExecutionState {
    fn from(s: woodstock::server::backup::save_state::BackupExecutionState) -> Self {
        use woodstock::server::backup::save_state::BackupExecutionState as Src;
        match s {
            Src::Waiting => BackupExecutionState::Waiting,
            Src::Skipped => BackupExecutionState::Skipped,
            Src::Authenticate => BackupExecutionState::Authenticate,
            Src::Initialization => BackupExecutionState::Initialization,
            Src::PreCommands(_) => BackupExecutionState::PreCommands,
            Src::DownloadFileList(_) => BackupExecutionState::DownloadFileList,
            Src::DownloadChunks(_) => BackupExecutionState::DownloadChunks,
            Src::PostCommands(_) => BackupExecutionState::PostCommands,
            Src::Compact(_) => BackupExecutionState::Compact,
            Src::CountReferences => BackupExecutionState::CountReferences,
            Src::AddReferencesToPool => BackupExecutionState::AddReferencesToPool,
            Src::Completed => BackupExecutionState::Completed,
        }
    }
}

/// Create backup request (REST)
#[derive(Debug, serde::Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateBackupRequest {
    pub host_name: String,
    pub backup_type: Option<String>,
}

/// Archive generation request (REST)
#[derive(Debug, serde::Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveRequest {
    pub files: Vec<String>,
    pub format: String,
}

// --- GraphQL: état détaillé d'une tâche de sauvegarde ---

#[derive(SimpleObject, Clone)]
pub struct JobBackupTaskState {
    pub execution_state: BackupExecutionState,
    pub error_message: Option<String>,
    pub progression: BackupProgression,
    pub pre_command_states: Vec<ExecuteCommandState>,
    pub share_states: Vec<ShareState>,
    pub post_command_states: Vec<ExecuteCommandState>,
    pub error_state: Option<BackupErrorState>,
}

impl From<woodstock::server::backup::save_state::BackupState> for JobBackupTaskState {
    fn from(s: woodstock::server::backup::save_state::BackupState) -> Self {
        // Extraire error_state une seule fois pour éviter clone
        let error_message = s
            .error_state
            .as_ref()
            .map(backup_error_state_message)
            .cloned();
        let error_state = s.error_state.map(BackupErrorState::from);

        JobBackupTaskState {
            execution_state: s.execution_state.into(),
            error_message,
            progression: s.global_progression.into(),
            share_states: s.share_states.into_values().map(Into::into).collect(),
            pre_command_states: s.pre_command_states.into_values().map(Into::into).collect(),
            post_command_states: s
                .post_command_states
                .into_values()
                .map(Into::into)
                .collect(),
            error_state,
        }
    }
}

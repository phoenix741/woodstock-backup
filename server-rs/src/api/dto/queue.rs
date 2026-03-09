use async_graphql::{Enum, InputObject, SimpleObject, Union};
use chrono::Local;

use crate::graphql::scalars::BigIntScalar;

use super::HostConfiguration;

#[derive(SimpleObject, Clone)]
pub struct JobResponse {
    pub id: String,
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq)]
#[graphql(remote = "crate::jobs::progress::JobStatus")]
pub enum JobStatus {
    Created,
    Started,
    Completed,
    Failed,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum JobKind {
    Backup,
    Restore,
    Remove,
    CleanupRefcnt,
    Fsck,
    Stats,
}

impl From<crate::jobs::progress::JobKind> for JobKind {
    fn from(kind: crate::jobs::progress::JobKind) -> Self {
        match kind {
            crate::jobs::progress::JobKind::Backup(_) => JobKind::Backup,
            crate::jobs::progress::JobKind::Restore(_) => JobKind::Restore,
            crate::jobs::progress::JobKind::Remove(_) => JobKind::Remove,
            crate::jobs::progress::JobKind::CleanupRefcnt(_) => JobKind::CleanupRefcnt,
            crate::jobs::progress::JobKind::Fsck(_) => JobKind::Fsck,
            crate::jobs::progress::JobKind::Stats(_) => JobKind::Stats,
        }
    }
}

#[derive(InputObject, Default, Debug)]
pub struct QueueListInput {
    #[graphql(default)]
    pub state: Option<JobStatus>,
    #[graphql(name = "queueName")]
    pub queue_name: Option<String>,
    #[graphql(name = "operationName")]
    pub operation_name: Option<String>,
}

impl From<QueueListInput> for crate::jobs::progress::ProgressFilter {
    fn from(input: QueueListInput) -> Self {
        Self {
            host: None,
            job_id: None,
            kind: input.operation_name,
            status: input.state.map(|s| s.into()),
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct JobBackupData {
    pub host: String,
    pub config: HostConfiguration,
    /// UUID v7 of this backup (primary key)
    pub id: String,
    /// Sequential display number
    pub number: usize,
    #[graphql(name = "previousId")]
    pub previous_id: Option<String>,
    pub ip: Option<String>,
    #[graphql(name = "startDate")]
    pub start_date: Option<chrono::DateTime<Local>>,
    pub force: bool,
}

impl From<crate::jobs::types::BackupJobData> for JobBackupData {
    fn from(data: crate::jobs::types::BackupJobData) -> Self {
        Self {
            host: data.host,
            config: data.config.into(),
            id: data.id.to_string(),
            number: data.number,
            previous_id: data.previous_id.map(|id| id.to_string()),
            ip: data.ip.map(|ip| ip.to_string()),
            start_date: data.start_date,
            force: data.force,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct JobRestoreDataSelection {
    pub share: String,
    pub selection: Vec<String>,
}
impl From<crate::jobs::types::JobRestoreDataSelection> for JobRestoreDataSelection {
    fn from(value: crate::jobs::types::JobRestoreDataSelection) -> Self {
        Self {
            share: value.share,
            selection: value.selection,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct JobRestoreData {
    pub host: String,
    pub config: Option<HostConfiguration>,
    /// UUID v7 of the backup to restore
    pub id: String,
    /// Sequential display number
    pub number: usize,
    pub ip: Option<String>,
    #[graphql(name = "startDate")]
    pub start_date: Option<chrono::DateTime<Local>>,
    #[graphql(name = "destinationDirectory")]
    pub destination_directory: String,
    pub files: Vec<JobRestoreDataSelection>,
}
impl From<crate::jobs::types::RestoreJobData> for JobRestoreData {
    fn from(data: crate::jobs::types::RestoreJobData) -> Self {
        Self {
            host: data.host,
            config: data.config.map(Into::into),
            id: data.id.to_string(),
            number: data.number,
            ip: data.ip.map(|ip| ip.to_string()),
            start_date: data.start_date,
            destination_directory: data.destination_directory,
            files: data.files.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct JobRemoveData {
    pub host: String,
    pub config: Option<HostConfiguration>,
    /// UUID v7 of the backup to remove
    pub id: String,
    /// Sequential display number
    pub number: usize,
    #[graphql(name = "startDate")]
    pub start_date: Option<chrono::DateTime<Local>>,
}
impl From<crate::jobs::types::RemoveJobData> for JobRemoveData {
    fn from(data: crate::jobs::types::RemoveJobData) -> Self {
        Self {
            host: data.host,
            config: data.config.map(Into::into),
            id: data.id.to_string(),
            number: data.number,
            start_date: data.start_date,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct JobCleanupData {
    pub target: Option<String>,
}
impl From<crate::jobs::types::CleanupRefcntJobData> for JobCleanupData {
    fn from(data: crate::jobs::types::CleanupRefcntJobData) -> Self {
        Self {
            target: data.target,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct JobFsckData {
    #[graphql(name = "dryRun")]
    pub dry_run: bool,
    #[graphql(name = "verifyChunks")]
    pub verify_chunks: bool,
}
impl From<crate::jobs::types::FsckJobData> for JobFsckData {
    fn from(data: crate::jobs::types::FsckJobData) -> Self {
        Self {
            dry_run: data.dry_run,
            verify_chunks: data.verify_chunks,
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum RestoreErrorState {
    AuthenticationError,
    PreparationError,
    RestoreError,
    Unknown,
}
impl From<woodstock::server::backup::restore_state::ErrorState> for RestoreErrorState {
    fn from(s: woodstock::server::backup::restore_state::ErrorState) -> Self {
        use woodstock::server::backup::restore_state::ErrorState as Src;
        match s {
            Src::AuthenticationError(_) => Self::AuthenticationError,
            Src::PreparationError(_) => Self::PreparationError,
            Src::RestoreError(_) => Self::RestoreError,
            Src::Unknown(_) => Self::Unknown,
        }
    }
}
fn restore_error_message(e: &woodstock::server::backup::restore_state::ErrorState) -> String {
    use woodstock::server::backup::restore_state::ErrorState as Src;
    match e {
        Src::AuthenticationError(m)
        | Src::PreparationError(m)
        | Src::RestoreError(m)
        | Src::Unknown(m) => m.clone(),
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum RestoreExecutionState {
    Waiting,
    Authentication,
    Preparation,
    Restoring,
    Completed,
}
impl From<woodstock::server::backup::restore_state::RestoreExecutionState>
    for RestoreExecutionState
{
    fn from(s: woodstock::server::backup::restore_state::RestoreExecutionState) -> Self {
        use woodstock::server::backup::restore_state::RestoreExecutionState as Src;
        match s {
            Src::Waiting => Self::Waiting,
            Src::Authentication => Self::Authentication,
            Src::Preparation(_) => Self::Preparation,
            Src::Restoring(_) => Self::Restoring,
            Src::Completed => Self::Completed,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct JobRestoreTaskState {
    pub execution_state: RestoreExecutionState,
    pub global_progression: super::BackupProgression,
    pub error_state: Option<RestoreErrorState>,
    pub error_message: Option<String>,
}
impl From<woodstock::server::backup::restore_state::RestoreState> for JobRestoreTaskState {
    fn from(s: woodstock::server::backup::restore_state::RestoreState) -> Self {
        // Extraire error_state une seule fois pour éviter clone
        let error_message = s.error_state.as_ref().map(restore_error_message);
        let error_state = s.error_state.map(RestoreErrorState::from);

        Self {
            execution_state: s.execution_state.into(),
            global_progression: s.global_progression.into(),
            error_state,
            error_message,
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum RemoveErrorState {
    AddReferencesToPoolError,
    RefcntRemovalError,
    BackupRemovalError,
    Unknown,
}
impl From<woodstock::server::backup::remove_state::ErrorState> for RemoveErrorState {
    fn from(s: woodstock::server::backup::remove_state::ErrorState) -> Self {
        use woodstock::server::backup::remove_state::ErrorState as Src;
        match s {
            Src::AddReferencesToPoolError(_) => Self::AddReferencesToPoolError,
            Src::RefcntRemovalError(_) => Self::RefcntRemovalError,
            Src::BackupRemovalError(_) => Self::BackupRemovalError,
            Src::Unknown(_) => Self::Unknown,
        }
    }
}
fn remove_error_message(e: &woodstock::server::backup::remove_state::ErrorState) -> String {
    use woodstock::server::backup::remove_state::ErrorState as Src;
    match e {
        Src::AddReferencesToPoolError(m)
        | Src::RefcntRemovalError(m)
        | Src::BackupRemovalError(m)
        | Src::Unknown(m) => m.clone(),
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum RemoveExecutionState {
    Waiting,
    AddReferencesToPool,
    RemovingRefcnt,
    RemovingBackup,
    Completed,
}
impl From<woodstock::server::backup::remove_state::RemoveExecutionState> for RemoveExecutionState {
    fn from(s: woodstock::server::backup::remove_state::RemoveExecutionState) -> Self {
        use woodstock::server::backup::remove_state::RemoveExecutionState as Src;
        match s {
            Src::Waiting => Self::Waiting,
            Src::AddReferencesToPool => Self::AddReferencesToPool,
            Src::RemovingRefcnt => Self::RemovingRefcnt,
            Src::RemovingBackup => Self::RemovingBackup,
            Src::Completed => Self::Completed,
        }
    }
}
#[derive(SimpleObject, Clone)]
pub struct JobRemoveState {
    pub execution_state: RemoveExecutionState,
    pub error_state: Option<RemoveErrorState>,
    pub error_message: Option<String>,
}
impl From<woodstock::server::backup::remove_state::RemoveState> for JobRemoveState {
    fn from(s: woodstock::server::backup::remove_state::RemoveState) -> Self {
        // Extraire error_state une seule fois pour éviter clone
        let error_message = s.error_state.as_ref().map(remove_error_message);
        let error_state = s.error_state.map(RemoveErrorState::from);

        JobRemoveState {
            execution_state: s.execution_state.into(),
            error_state,
            error_message,
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum FsckErrorState {
    ApplyingRefcntError,
    InitializationError,
    VerifyRefcntError,
    VerifyUnusedError,
    VerifyChunkError,
    Unknown,
}
impl From<woodstock::server::pool::fsck_state::ErrorState> for FsckErrorState {
    fn from(s: woodstock::server::pool::fsck_state::ErrorState) -> Self {
        use woodstock::server::pool::fsck_state::ErrorState as Src;
        match s {
            Src::ApplyingRefcntError(_) => Self::ApplyingRefcntError,
            Src::InitializationError(_) => Self::InitializationError,
            Src::VerifyRefcntError(_) => Self::VerifyRefcntError,
            Src::VerifyUnusedError(_) => Self::VerifyUnusedError,
            Src::VerifyChunkError(_) => Self::VerifyChunkError,
            Src::Unknown(_) => Self::Unknown,
        }
    }
}
fn fsck_error_message(e: &woodstock::server::pool::fsck_state::ErrorState) -> String {
    use woodstock::server::pool::fsck_state::ErrorState as Src;
    match e {
        Src::ApplyingRefcntError(m)
        | Src::InitializationError(m)
        | Src::VerifyRefcntError(m)
        | Src::VerifyUnusedError(m)
        | Src::VerifyChunkError(m)
        | Src::Unknown(m) => m.clone(),
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum FsckExecutionState {
    Waiting,
    ApplyingRefcnt,
    Initialization,
    VerifyRefcnt,
    VerifyUnused,
    VerifyChunk,
    Completed,
}
impl From<woodstock::server::pool::fsck_state::FsckExecutionState> for FsckExecutionState {
    fn from(s: woodstock::server::pool::fsck_state::FsckExecutionState) -> Self {
        use woodstock::server::pool::fsck_state::FsckExecutionState as Src;
        match s {
            Src::Waiting => Self::Waiting,
            Src::ApplyingRefcnt => Self::ApplyingRefcnt,
            Src::Initialization => Self::Initialization,
            Src::VerifyRefcnt => Self::VerifyRefcnt,
            Src::VerifyUnused => Self::VerifyUnused,
            Src::VerifyChunk => Self::VerifyChunk,
            Src::Completed => Self::Completed,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct RefcntProgression {
    pub progress_max: usize,
    pub progress_current: usize,
    pub error_count: usize,
    pub total_count: usize,
}
impl From<woodstock::server::pool::fsck_state::RefcntProgression> for RefcntProgression {
    fn from(p: woodstock::server::pool::fsck_state::RefcntProgression) -> Self {
        Self {
            progress_max: p.progress_max,
            progress_current: p.progress_current,
            error_count: p.error_count,
            total_count: p.total_count,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct UnusedProgression {
    pub progress_max: usize,
    pub progress_current: usize,
    pub in_nothing: usize,
    pub in_refcnt: usize,
    pub in_unused: usize,
    pub missing: usize,
}
impl From<woodstock::server::pool::fsck_state::UnusedProgression> for UnusedProgression {
    fn from(p: woodstock::server::pool::fsck_state::UnusedProgression) -> Self {
        Self {
            progress_max: p.progress_max,
            progress_current: p.progress_current,
            in_nothing: p.in_nothing,
            in_refcnt: p.in_refcnt,
            in_unused: p.in_unused,
            missing: p.missing,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct ChunkProgression {
    pub progress_max: usize,
    pub progress_current: usize,
    pub error_count: usize,
    pub total_count: usize,
}
impl From<woodstock::server::pool::fsck_state::ChunkProgression> for ChunkProgression {
    fn from(p: woodstock::server::pool::fsck_state::ChunkProgression) -> Self {
        Self {
            progress_max: p.progress_max,
            progress_current: p.progress_current,
            error_count: p.error_count,
            total_count: p.total_count,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct JobFsckTaskState {
    pub execution_state: FsckExecutionState,
    pub error_state: Option<FsckErrorState>,
    pub error_message: Option<String>,
    pub refcnt_progression: RefcntProgression,
    pub unused_progression: UnusedProgression,
    pub chunk_progression: ChunkProgression,
    pub dry_run: bool,
}
impl From<woodstock::server::pool::fsck_state::FsckState> for JobFsckTaskState {
    fn from(s: woodstock::server::pool::fsck_state::FsckState) -> Self {
        // Extraire error_state une seule fois pour éviter clone
        let error_message = s.error_state.as_ref().map(fsck_error_message);
        let error_state = s.error_state.map(FsckErrorState::from);

        Self {
            execution_state: s.execution_state.into(),
            error_state,
            error_message,
            refcnt_progression: s.refcnt_progression.into(),
            unused_progression: s.unused_progression.into(),
            chunk_progression: s.chunk_progression.into(),
            dry_run: s.dry_run,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct JobCleanerTaskState {
    pub execution_state: CleanerExecutionState,
    pub error_state: Option<CleanerErrorState>,
    pub error_message: Option<String>,
    pub progression: CleanerProgression,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum CleanerErrorState {
    ApplyingRefcntError,
    InitializationError,
    CleaningError,
    Unknown,
}
impl From<woodstock::server::pool::pool_cleaner_state::ErrorState> for CleanerErrorState {
    fn from(s: woodstock::server::pool::pool_cleaner_state::ErrorState) -> Self {
        use woodstock::server::pool::pool_cleaner_state::ErrorState as Src;
        match s {
            Src::ApplyingRefcntError(_) => Self::ApplyingRefcntError,
            Src::InitializationError(_) => Self::InitializationError,
            Src::CleaningError(_) => Self::CleaningError,
            Src::Unknown(_) => Self::Unknown,
        }
    }
}
fn cleaner_error_message(e: &woodstock::server::pool::pool_cleaner_state::ErrorState) -> String {
    use woodstock::server::pool::pool_cleaner_state::ErrorState as Src;
    match e {
        Src::ApplyingRefcntError(m)
        | Src::InitializationError(m)
        | Src::CleaningError(m)
        | Src::Unknown(m) => m.clone(),
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum CleanerExecutionState {
    Waiting,
    ApplyingRefcnt,
    Initialization,
    Cleaning,
    Completed,
}
impl From<woodstock::server::pool::pool_cleaner_state::CleanerExecutionState>
    for CleanerExecutionState
{
    fn from(s: woodstock::server::pool::pool_cleaner_state::CleanerExecutionState) -> Self {
        use woodstock::server::pool::pool_cleaner_state::CleanerExecutionState as Src;
        match s {
            Src::Waiting => Self::Waiting,
            Src::ApplyingRefcnt => Self::ApplyingRefcnt,
            Src::Initialization => Self::Initialization,
            Src::Cleaning => Self::Cleaning,
            Src::Completed => Self::Completed,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct CleanerProgression {
    pub progress_max: usize,
    pub progress_current: usize,
    pub file_size: BigIntScalar,
    pub compressed_file_size: BigIntScalar,
}
impl From<woodstock::server::pool::pool_cleaner_state::CleanerProgression> for CleanerProgression {
    fn from(p: woodstock::server::pool::pool_cleaner_state::CleanerProgression) -> Self {
        Self {
            progress_max: p.progress_max,
            progress_current: p.progress_current,
            file_size: BigIntScalar(p.file_size),
            compressed_file_size: BigIntScalar(p.compressed_file_size),
        }
    }
}
impl From<woodstock::server::pool::pool_cleaner_state::CleanerState> for JobCleanerTaskState {
    fn from(s: woodstock::server::pool::pool_cleaner_state::CleanerState) -> Self {
        // Extraire error_state une seule fois pour éviter clone
        let error_message = s.error_state.as_ref().map(cleaner_error_message);
        let error_state = s.error_state.map(CleanerErrorState::from);

        Self {
            execution_state: s.execution_state.into(),
            error_state,
            error_message,
            progression: s.progression.into(),
        }
    }
}

#[derive(Union, Clone)]
pub enum BackupQueueProgress {
    JobBackupTaskState(crate::api::dto::JobBackupTaskState),
    JobRestoreTaskState(JobRestoreTaskState),
    JobRemoveState(JobRemoveState),
    JobCleanerTaskState(JobCleanerTaskState),
    JobFsckTaskState(JobFsckTaskState),
}
impl From<crate::jobs::progress::JobKind> for Option<BackupQueueProgress> {
    fn from(kind: crate::jobs::progress::JobKind) -> Self {
        match kind {
            crate::jobs::progress::JobKind::Backup(pd) => pd
                .progress
                .map(|p| BackupQueueProgress::JobBackupTaskState(p.into())),
            crate::jobs::progress::JobKind::Restore(pd) => pd
                .progress
                .map(|p| BackupQueueProgress::JobRestoreTaskState(p.into())),
            crate::jobs::progress::JobKind::Remove(pd) => pd
                .progress
                .map(|p| BackupQueueProgress::JobRemoveState(p.into())),
            crate::jobs::progress::JobKind::CleanupRefcnt(pd) => pd
                .progress
                .map(|p| BackupQueueProgress::JobCleanerTaskState(p.into())),
            crate::jobs::progress::JobKind::Fsck(pd) => pd
                .progress
                .map(|p| BackupQueueProgress::JobFsckTaskState(p.into())),
            crate::jobs::progress::JobKind::Stats(_) => None,
        }
    }
}

#[derive(Union, Clone)]
pub enum BackupQueueData {
    JobBackupData(JobBackupData),
    JobRestoreData(JobRestoreData),
    JobRemoveData(JobRemoveData),
    JobCleanupData(JobCleanupData),
    JobFsckData(JobFsckData),
    JobStatsData(JobStatsData),
}

#[derive(SimpleObject, Clone)]
pub struct JobStatsData {
    pub empty: bool,
}
impl Default for JobStatsData {
    fn default() -> Self {
        Self { empty: true }
    }
}
impl From<crate::jobs::types::StatsJobData> for JobStatsData {
    fn from(_: crate::jobs::types::StatsJobData) -> Self {
        JobStatsData { empty: true }
    }
}

#[derive(SimpleObject, Clone)]
pub struct Job {
    pub job_id: String,
    pub kind: JobKind,
    pub status: JobStatus,
    pub timestamp: i64,
    pub host: Option<String>,
    pub data: BackupQueueData,
    pub progress: Option<BackupQueueProgress>,
    #[graphql(name = "failedReason")]
    pub failed_reason: Option<String>,
}
impl From<crate::jobs::progress::ProgressEvent> for Job {
    fn from(event: crate::jobs::progress::ProgressEvent) -> Self {
        // Déconstruction unique pour éviter les clones multiples coûteux
        let (kind_enum, data, progress) = match event.kind {
            crate::jobs::progress::JobKind::Backup(pd) => (
                JobKind::Backup,
                BackupQueueData::JobBackupData(pd.data.into()),
                pd.progress
                    .map(|p| BackupQueueProgress::JobBackupTaskState(p.into())),
            ),
            crate::jobs::progress::JobKind::Restore(pd) => (
                JobKind::Restore,
                BackupQueueData::JobRestoreData(pd.data.into()),
                pd.progress
                    .map(|p| BackupQueueProgress::JobRestoreTaskState(p.into())),
            ),
            crate::jobs::progress::JobKind::Remove(pd) => (
                JobKind::Remove,
                BackupQueueData::JobRemoveData(pd.data.into()),
                pd.progress
                    .map(|p| BackupQueueProgress::JobRemoveState(p.into())),
            ),
            crate::jobs::progress::JobKind::CleanupRefcnt(pd) => (
                JobKind::CleanupRefcnt,
                BackupQueueData::JobCleanupData(pd.data.into()),
                pd.progress
                    .map(|p| BackupQueueProgress::JobCleanerTaskState(p.into())),
            ),
            crate::jobs::progress::JobKind::Fsck(pd) => (
                JobKind::Fsck,
                BackupQueueData::JobFsckData(pd.data.into()),
                pd.progress
                    .map(|p| BackupQueueProgress::JobFsckTaskState(p.into())),
            ),
            crate::jobs::progress::JobKind::Stats(pd) => (
                JobKind::Stats,
                BackupQueueData::JobStatsData(pd.data.into()),
                None,
            ),
        };

        Job {
            job_id: event.job_id,
            kind: kind_enum,
            status: event.status.into(),
            timestamp: event.timestamp,
            host: event.host,
            data,
            progress,
            failed_reason: event.failed_reason,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct QueueStats {
    pub pending: usize,
    pub running: usize,
    pub success: usize,
    pub failed: usize,
    pub dead: usize,
    pub last_execution: Option<chrono::DateTime<Local>>,
    pub next_wakeup: Option<chrono::DateTime<Local>>,
}

#[derive(InputObject)]
pub struct RestoreFilesInput {
    pub share: String,
    pub selection: Vec<String>,
}
#[derive(InputObject)]
pub struct RestoreInput {
    pub hostname: String,
    /// UUID v7 of the backup to restore
    pub id: String,
    pub destination_directory: String,
    pub files: Vec<RestoreFilesInput>,
}

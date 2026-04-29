use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use async_graphql::{Enum, ID};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ClientType {
    Windows,
    Linux,
    LinuxDeb,
    LinuxLite,
    #[default]
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct HostConfiguration {
    pub addresses: Option<Vec<String>>,
    pub port: u16,
    pub operations: HostConfigOperation,
    pub schedule: Option<Schedule>,
}

impl From<woodstock::config::HostConfiguration> for HostConfiguration {
    fn from(config: woodstock::config::HostConfiguration) -> Self {
        Self {
            addresses: config.addresses,
            port: config.port,
            operations: HostConfigOperation::from(config.operations),
            schedule: config.schedule.map(|s| Schedule::from(s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct HostConfigOperation {
    pub pre_commands: Option<Vec<ExecuteCommandOperation>>,
    pub operation: Option<BackupOperation>,
    pub post_commands: Option<Vec<ExecuteCommandOperation>>,
}

impl From<woodstock::config::HostConfigOperation> for HostConfigOperation {
    fn from(operations: woodstock::config::HostConfigOperation) -> Self {
        Self {
            pre_commands: operations.pre_commands.map(|cmds| {
                cmds.into_iter()
                    .map(ExecuteCommandOperation::from)
                    .collect()
            }),
            operation: operations.operation.map(BackupOperation::from),
            post_commands: operations.post_commands.map(|cmds| {
                cmds.into_iter()
                    .map(ExecuteCommandOperation::from)
                    .collect()
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, SimpleObject)]
pub struct ExecuteCommandOperation {
    pub command: String,
}

impl From<woodstock::config::ExecuteCommandOperation> for ExecuteCommandOperation {
    fn from(cmd: woodstock::config::ExecuteCommandOperation) -> Self {
        Self {
            command: cmd.command,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct BackupOperation {
    pub shares: Vec<BackupTaskShare>,
    pub includes: Option<Vec<String>>,
    pub excludes: Option<Vec<String>>,
    pub timeout: Option<u64>,
}

impl From<woodstock::config::BackupOperation> for BackupOperation {
    fn from(operation: woodstock::config::BackupOperation) -> Self {
        Self {
            shares: operation
                .shares
                .into_iter()
                .map(BackupTaskShare::from)
                .collect(),
            includes: operation.includes,
            excludes: operation.excludes,
            timeout: operation.timeout,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, SimpleObject)]
pub struct BackupTaskShare {
    pub name: String,
    pub includes: Option<Vec<String>>,
    pub excludes: Option<Vec<String>>,
}

impl From<woodstock::config::BackupTaskShare> for BackupTaskShare {
    fn from(share: woodstock::config::BackupTaskShare) -> Self {
        Self {
            name: share.name,
            includes: share.includes,
            excludes: share.excludes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub activated: Option<bool>,
    pub backup_period: Option<i64>,
    pub backup_to_keep: Option<ScheduledBackupToKeep>,
}

impl From<woodstock::config::Schedule> for Schedule {
    fn from(schedule: woodstock::config::Schedule) -> Self {
        Self {
            activated: schedule.activated,
            backup_period: schedule.backup_period,
            backup_to_keep: schedule.backup_to_keep.map(ScheduledBackupToKeep::from),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, SimpleObject)]
pub struct ScheduledBackupToKeep {
    pub hourly: Option<usize>,
    pub daily: Option<usize>,
    pub weekly: Option<usize>,
    pub monthly: Option<usize>,
    pub yearly: Option<usize>,
}

impl From<woodstock::config::ScheduledBackupToKeep> for ScheduledBackupToKeep {
    fn from(keep: woodstock::config::ScheduledBackupToKeep) -> Self {
        Self {
            hourly: keep.hourly,
            daily: keep.daily,
            weekly: keep.weekly,
            monthly: keep.monthly,
            yearly: keep.yearly,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HostInformation {
    pub name: String,
    pub last_backup: Option<crate::api::dto::Backup>,
}

#[derive(async_graphql::SimpleObject, Clone)]
#[graphql(complex)]
pub struct Host {
    pub name: ID,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum HostAvailibilityState {
    Online,
    Offline,
    Unknown,
}

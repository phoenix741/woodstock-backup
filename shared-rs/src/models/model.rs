// ************ Schedule ************

use std::collections::HashMap;

use napi::bindgen_prelude::{BigInt, Buffer};

#[napi(object)]
pub struct JsScheduledBackupToKeep {
  pub hourly: Option<u8>,
  pub daily: Option<u8>,
  pub weekly: Option<u8>,
  pub monthly: Option<u8>,
  pub yearly: Option<u8>,
}

impl From<woodstock::config::ScheduledBackupToKeep> for JsScheduledBackupToKeep {
  fn from(backup_to_keep: woodstock::config::ScheduledBackupToKeep) -> Self {
    Self {
      hourly: backup_to_keep.hourly,
      daily: backup_to_keep.daily,
      weekly: backup_to_keep.weekly,
      monthly: backup_to_keep.monthly,
      yearly: backup_to_keep.yearly,
    }
  }
}

#[napi(object)]
pub struct JsSchedule {
  pub activated: Option<bool>,
  pub backup_period: Option<u8>,
  pub backup_to_keep: Option<JsScheduledBackupToKeep>,
}

impl From<woodstock::config::Schedule> for JsSchedule {
  fn from(schedule: woodstock::config::Schedule) -> Self {
    Self {
      activated: schedule.activated,
      backup_period: schedule.backup_period,
      backup_to_keep: schedule.backup_to_keep.map(std::convert::Into::into),
    }
  }
}

// ************* Host **************

#[napi(object)]
pub struct JsDhcpAddress {
  pub address: String,
  pub start: u8,
  pub end: u8,
}

impl From<woodstock::config::DhcpAddress> for JsDhcpAddress {
  fn from(dhcp_address: woodstock::config::DhcpAddress) -> Self {
    Self {
      address: dhcp_address.address,
      start: dhcp_address.start,
      end: dhcp_address.end,
    }
  }
}

#[napi(object)]
pub struct JsBackupTaskShare {
  pub name: String,
  pub includes: Option<Vec<String>>,
  pub excludes: Option<Vec<String>>,
}

impl From<woodstock::config::BackupTaskShare> for JsBackupTaskShare {
  fn from(backup_task_share: woodstock::config::BackupTaskShare) -> Self {
    Self {
      name: backup_task_share.name,
      includes: backup_task_share.includes,
      excludes: backup_task_share.excludes,
    }
  }
}

#[napi(object)]
pub struct JsExecuteCommandOperation {
  pub command: String,
}

impl From<woodstock::config::ExecuteCommandOperation> for JsExecuteCommandOperation {
  fn from(execute_command_operation: woodstock::config::ExecuteCommandOperation) -> Self {
    Self {
      command: execute_command_operation.command,
    }
  }
}

#[napi(object)]
pub struct JsBackupOperation {
  pub shares: Vec<JsBackupTaskShare>,
  pub includes: Option<Vec<String>>,
  pub excludes: Option<Vec<String>>,
  pub timeout: Option<i64>,
}

impl From<woodstock::config::BackupOperation> for JsBackupOperation {
  fn from(backup_operation: woodstock::config::BackupOperation) -> Self {
    Self {
      shares: backup_operation
        .shares
        .into_iter()
        .map(std::convert::Into::into)
        .collect(),
      includes: backup_operation.includes,
      excludes: backup_operation.excludes,
      timeout: backup_operation.timeout.map(|t| i64::try_from(t).unwrap()),
    }
  }
}

#[napi(object)]
pub struct JsHostConfigOperation {
  pub pre_commands: Option<Vec<JsExecuteCommandOperation>>,
  pub operation: Option<JsBackupOperation>,
  pub post_commands: Option<Vec<JsExecuteCommandOperation>>,
}

impl From<woodstock::config::HostConfigOperation> for JsHostConfigOperation {
  fn from(host_config_operation: woodstock::config::HostConfigOperation) -> Self {
    Self {
      pre_commands: host_config_operation
        .pre_commands
        .map(|c| c.into_iter().map(std::convert::Into::into).collect()),
      operation: host_config_operation
        .operation
        .map(std::convert::Into::into),
      post_commands: host_config_operation
        .post_commands
        .map(|c| c.into_iter().map(std::convert::Into::into).collect()),
    }
  }
}

#[napi(object)]
pub struct JsHostConfiguration {
  pub is_local: Option<bool>,
  pub password: String,
  pub addresses: Option<Vec<String>>,
  pub dhcp: Option<Vec<JsDhcpAddress>>,
  pub operations: JsHostConfigOperation,
  pub schedule: Option<JsSchedule>,
}

impl From<woodstock::config::HostConfiguration> for JsHostConfiguration {
  fn from(host_configuration: woodstock::config::HostConfiguration) -> Self {
    Self {
      is_local: host_configuration.is_local,
      password: host_configuration.password,
      addresses: host_configuration.addresses,
      dhcp: host_configuration
        .dhcp
        .map(|d| d.into_iter().map(std::convert::Into::into).collect()),
      operations: host_configuration.operations.into(),
      schedule: host_configuration.schedule.map(std::convert::Into::into),
    }
  }
}

// ************ Backup ****************

#[napi(object)]
pub struct JsBackup {
  pub number: u32,
  pub completed: bool,

  pub start_date: i64,
  pub end_date: Option<i64>,

  pub error_count: u32,

  pub file_count: u32,
  pub new_file_count: u32,
  pub removed_file_count: u32,
  pub modified_file_count: u32,
  pub existing_file_count: u32,

  pub file_size: BigInt,
  pub new_file_size: BigInt,
  pub modified_file_size: BigInt,
  pub existing_file_size: BigInt,

  pub compressed_file_size: BigInt,
  pub new_compressed_file_size: BigInt,
  pub modified_compressed_file_size: BigInt,
  pub existing_compressed_file_size: BigInt,

  pub speed: f64,
}

impl From<woodstock::config::Backup> for JsBackup {
  fn from(backup: woodstock::config::Backup) -> Self {
    Self {
      number: u32::try_from(backup.number).unwrap(),
      completed: backup.completed,

      start_date: i64::try_from(backup.start_date).unwrap_or_default(),
      end_date: backup
        .end_date
        .map(|d| i64::try_from(d).unwrap_or_default()),

      error_count: u32::try_from(backup.error_count).unwrap(),

      file_count: u32::try_from(backup.file_count).unwrap(),
      new_file_count: u32::try_from(backup.new_file_count).unwrap(),
      removed_file_count: u32::try_from(backup.removed_file_count).unwrap(),
      modified_file_count: u32::try_from(backup.modified_file_count).unwrap(),
      existing_file_count: u32::try_from(backup.existing_file_count).unwrap(),

      file_size: backup.file_size.into(),
      new_file_size: backup.new_file_size.into(),
      modified_file_size: backup.modified_file_size.into(),
      existing_file_size: backup.existing_file_size.into(),

      compressed_file_size: backup.compressed_file_size.into(),
      new_compressed_file_size: backup.new_compressed_file_size.into(),
      modified_compressed_file_size: backup.modified_compressed_file_size.into(),
      existing_compressed_file_size: backup.existing_compressed_file_size.into(),

      speed: backup.speed,
    }
  }
}

#[napi]
pub enum JsFileManifestType {
  RegularFile = 0,
  Symlink = 1,
  Directory = 2,
  BlockDevice = 3,
  CharacterDevice = 4,
  Fifo = 5,
  Socket = 6,
  Unknown = 99,
}

impl From<woodstock::FileManifestType> for JsFileManifestType {
  fn from(file_manifest_type: woodstock::FileManifestType) -> Self {
    match file_manifest_type {
      woodstock::FileManifestType::RegularFile => Self::RegularFile,
      woodstock::FileManifestType::Symlink => Self::Symlink,
      woodstock::FileManifestType::Directory => Self::Directory,
      woodstock::FileManifestType::BlockDevice => Self::BlockDevice,
      woodstock::FileManifestType::CharacterDevice => Self::CharacterDevice,
      woodstock::FileManifestType::Fifo => Self::Fifo,
      woodstock::FileManifestType::Socket => Self::Socket,
      woodstock::FileManifestType::Unknown => Self::Unknown,
    }
  }
}

#[napi(object)]
pub struct JsFileManifestStat {
  pub owner_id: u32,
  pub group_id: u32,
  pub size: BigInt,
  pub compressed_size: BigInt,
  pub last_read: i64,
  pub last_modified: i64,
  pub created: i64,
  pub mode: u32,
  pub r#type: JsFileManifestType,
  pub dev: BigInt,
  pub rdev: BigInt,
  pub ino: BigInt,
  pub nlink: BigInt,
}

impl From<woodstock::FileManifestStat> for JsFileManifestStat {
  fn from(file_manifest_stat: woodstock::FileManifestStat) -> Self {
    Self {
      owner_id: file_manifest_stat.owner_id,
      group_id: file_manifest_stat.group_id,
      size: file_manifest_stat.size.into(),
      compressed_size: file_manifest_stat.compressed_size.into(),
      last_read: file_manifest_stat.last_read,
      last_modified: file_manifest_stat.last_modified,
      created: file_manifest_stat.created,
      mode: file_manifest_stat.mode,
      r#type: file_manifest_stat.r#type().into(),
      dev: file_manifest_stat.dev.into(),
      rdev: file_manifest_stat.rdev.into(),
      ino: file_manifest_stat.ino.into(),
      nlink: file_manifest_stat.nlink.into(),
    }
  }
}

#[napi]
pub enum JsFileManifestAclQualifier {
  Undefined = 0,
  UserObj = 1,
  GroupObj = 2,
  Other = 3,
  UserId = 4,
  GroupId = 5,
  Mask = 6,
}

impl From<woodstock::FileManifestAclQualifier> for JsFileManifestAclQualifier {
  fn from(file_manifest_acl_qualifier: woodstock::FileManifestAclQualifier) -> Self {
    match file_manifest_acl_qualifier {
      woodstock::FileManifestAclQualifier::Undefined => Self::Undefined,
      woodstock::FileManifestAclQualifier::UserObj => Self::UserObj,
      woodstock::FileManifestAclQualifier::GroupObj => Self::GroupObj,
      woodstock::FileManifestAclQualifier::Other => Self::Other,
      woodstock::FileManifestAclQualifier::UserId => Self::UserId,
      woodstock::FileManifestAclQualifier::GroupId => Self::GroupId,
      woodstock::FileManifestAclQualifier::Mask => Self::Mask,
    }
  }
}

#[napi(object)]
pub struct JsFileManifestAcl {
  pub qualifier: JsFileManifestAclQualifier,
  pub id: u32,
  pub perm: u32,
}

impl From<woodstock::FileManifestAcl> for JsFileManifestAcl {
  fn from(file_manifest_acl: woodstock::FileManifestAcl) -> Self {
    Self {
      qualifier: file_manifest_acl.qualifier().into(),
      id: file_manifest_acl.id,
      perm: file_manifest_acl.perm,
    }
  }
}

#[napi(object)]
pub struct JsFileManifestXAttr {
  pub key: Buffer,
  pub value: Buffer,
}

impl From<woodstock::FileManifestXAttr> for JsFileManifestXAttr {
  fn from(file_manifest_xattr: woodstock::FileManifestXAttr) -> Self {
    Self {
      key: file_manifest_xattr.key.into(),
      value: file_manifest_xattr.value.into(),
    }
  }
}

#[napi(object)]
pub struct JsFileManifest {
  pub path: Buffer,
  pub stats: Option<JsFileManifestStat>,
  pub symlink: Buffer,
  pub xattr: Vec<JsFileManifestXAttr>,
  pub acl: Vec<JsFileManifestAcl>,
  pub chunks: Vec<Buffer>,
  pub hash: Buffer,
  pub metadata: HashMap<String, Buffer>,
}

impl From<woodstock::FileManifest> for JsFileManifest {
  fn from(file_manifest: woodstock::FileManifest) -> Self {
    Self {
      path: file_manifest.path.into(),
      stats: file_manifest.stats.map(std::convert::Into::into),
      symlink: file_manifest.symlink.into(),
      xattr: file_manifest
        .xattr
        .into_iter()
        .map(std::convert::Into::into)
        .collect(),
      acl: file_manifest
        .acl
        .into_iter()
        .map(std::convert::Into::into)
        .collect(),
      chunks: file_manifest
        .chunks
        .into_iter()
        .map(std::convert::Into::into)
        .collect(),
      hash: file_manifest.hash.into(),
      metadata: file_manifest
        .metadata
        .into_iter()
        .map(|(key, vec)| (key, vec.into()))
        .collect(),
    }
  }
}

#[napi]
pub enum JsEntryType {
  Add = 0,
  Modify = 1,
  Remove = 2,
}

impl From<woodstock::EntryType> for JsEntryType {
  fn from(entry_type: woodstock::EntryType) -> Self {
    match entry_type {
      woodstock::EntryType::Add => Self::Add,
      woodstock::EntryType::Modify => Self::Modify,
      woodstock::EntryType::Remove => Self::Remove,
    }
  }
}

/// Journal entry
#[napi(object)]
pub struct JsFileManifestJournalEntry {
  pub r#type: JsEntryType,
  pub manifest: Option<JsFileManifest>,
}

impl From<woodstock::FileManifestJournalEntry> for JsFileManifestJournalEntry {
  fn from(file_manifest_journal_entry: woodstock::FileManifestJournalEntry) -> Self {
    Self {
      r#type: file_manifest_journal_entry.r#type().into(),
      manifest: file_manifest_journal_entry
        .manifest
        .map(std::convert::Into::into),
    }
  }
}

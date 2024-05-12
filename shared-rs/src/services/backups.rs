use std::time::SystemTime;

use napi::{Error, Result};
use woodstock::{
  config::{Backups, Context},
  pool::{Refcnt, RefcntApplySens},
};

use crate::{config::context::JsBackupContext, models::JsBackup};

#[napi(js_name = "CoreBackupsService")]
pub struct JsBackupsService {
  context: Context,
  backups: Backups,
}

#[napi]
impl JsBackupsService {
  #[napi(constructor)]
  #[must_use]
  pub fn new(context: &JsBackupContext) -> Self {
    let context: Context = context.into();

    Self {
      context: context.clone(),
      backups: Backups::new(&context),
    }
  }

  #[napi]
  pub fn get_backup_destination_directory(
    &self,
    hostname: String,
    backup_number: u32,
  ) -> Result<String> {
    let backup_number = usize::try_from(backup_number)
      .map_err(|_| Error::from_reason("Backup number is too large".to_string()))?;
    let path = self
      .backups
      .get_backup_destination_directory(&hostname, backup_number);
    Ok(path.to_string_lossy().to_string())
  }

  #[napi]
  pub fn get_log_directory(&self, hostname: String, backup_number: u32) -> Result<String> {
    let backup_number = usize::try_from(backup_number)
      .map_err(|_| Error::from_reason("Backup number is too large".to_string()))?;
    let path = self.backups.get_log_directory(&hostname, backup_number);
    Ok(path.to_string_lossy().to_string())
  }

  #[napi]
  pub fn get_host_path(&self, hostname: String) -> Result<String> {
    let path = self.backups.get_host_path(&hostname);
    Ok(path.to_string_lossy().to_string())
  }

  #[napi]
  pub async fn get_backup(&self, hostname: String, backup_number: u32) -> Result<Option<JsBackup>> {
    let backup_number = usize::try_from(backup_number)
      .map_err(|_| Error::from_reason("Backup number is too large".to_string()))?;
    let backup = self.backups.get_backup(&hostname, backup_number).await;
    Ok(backup.map(std::convert::Into::into))
  }

  #[napi]
  pub async fn get_backups(&self, hostname: String) -> Result<Vec<JsBackup>> {
    let backups = self.backups.get_backups(&hostname).await;
    Ok(backups.into_iter().map(std::convert::Into::into).collect())
  }

  #[napi]
  pub async fn get_last_backup(&self, hostname: String) -> Result<Option<JsBackup>> {
    let backup = self.backups.get_last_backup(&hostname).await;
    Ok(backup.map(std::convert::Into::into))
  }

  #[napi]
  pub async fn get_previous_backup(
    &self,
    hostname: String,
    backup_number: u32,
  ) -> Result<Option<JsBackup>> {
    let backup_number = usize::try_from(backup_number)
      .map_err(|_| Error::from_reason("Backup number is too large".to_string()))?;
    let backup = self
      .backups
      .get_previous_backup(&hostname, backup_number)
      .await;
    Ok(backup.map(std::convert::Into::into))
  }

  #[napi]
  pub async fn get_backup_share_paths(
    &self,
    hostname: String,
    backup_number: u32,
  ) -> Result<Vec<String>> {
    let backup_number = usize::try_from(backup_number)
      .map_err(|_| Error::from_reason("Backup number is too large".to_string()))?;
    let share_paths = self
      .backups
      .get_backup_share_paths(&hostname, backup_number)
      .await;
    Ok(share_paths)
  }

  #[napi]
  pub async fn remove_backup(&self, hostname: String, backup_number: u32) -> Result<JsBackup> {
    let backup_number = usize::try_from(backup_number)
      .map_err(|_| Error::from_reason("Backup number is too large".to_string()))?;
    let backup = self
      .backups
      .remove_backup(&hostname, backup_number)
      .await
      .map_err(|_| Error::from_reason("Failed to remove backup".to_string()))?;
    Ok(backup.into())
  }

  #[napi]
  pub async fn remove_refcnt_of_host(&self, hostname: String, backup_number: u32) -> Result<()> {
    let backup_number = usize::try_from(backup_number)
      .map_err(|_| Error::from_reason("Backup number is too large".to_string()))?;
    let from_directory = self
      .backups
      .get_backup_destination_directory(&hostname, backup_number);

    let host_directory = self.backups.get_host_path(&hostname);

    let mut backup_refcnt = Refcnt::new(&from_directory);
    backup_refcnt.load_refcnt(false).await;

    Refcnt::apply_all_from(
      &host_directory,
      &backup_refcnt,
      &RefcntApplySens::Decrease,
      &SystemTime::now(),
      &self.context,
    )
    .await
    .map_err(|_| Error::from_reason("Failed to remove refcnt of host".to_string()))?;

    Ok(())
  }
}

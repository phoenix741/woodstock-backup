use futures_util::StreamExt;
use napi::{
  threadsafe_function::{
    ErrorStrategy::{self},
    ThreadsafeFunction,
  },
  Error, JsFunction, Result,
};
use woodstock::{
  config::{Backups, Context},
  proto::ProtobufReader,
  FileManifestJournalEntry,
};

use crate::{config::context::JsBackupContext, models::JsBackup, server::AbortHandle};

#[napi(object)]
pub struct JsBaskupsLog {
  pub progress: Option<String>,
  pub error: Option<String>,
  pub complete: bool,
}

#[napi(js_name = "CoreBackupsService")]
pub struct JsBackupsService {
  backups: Backups,
}

#[napi]
impl JsBackupsService {
  #[napi(constructor)]
  #[must_use]
  pub fn new(context: &JsBackupContext) -> Self {
    let context: Context = context.into();

    Self {
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
  pub fn read_log(
    &self,
    hostname: String,
    backup_number: u32,
    share_path: String,
    #[napi(ts_arg_type = "(result: JsBaskupsLog) => void")] callback: JsFunction,
  ) -> Result<AbortHandle> {
    let tsfn: ThreadsafeFunction<JsBaskupsLog, ErrorStrategy::Fatal> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

    let manifest = self
      .backups
      .get_manifest(&hostname, backup_number as usize, &share_path);
    let log_path = manifest.log_path;

    let handle = tokio::spawn(async move {
      let Ok(mut messages) = ProtobufReader::<FileManifestJournalEntry>::new(&log_path, true).await
      else {
        tsfn.call(
          JsBaskupsLog {
            progress: None,
            error: Some(format!("Could not read {}", log_path.display()).to_string()),
            complete: true,
          },
          napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
        );
        return;
      };

      let mut messages = messages.into_stream();

      while let Some(message) = messages.next().await {
        let Ok(message) = message else {
          tsfn.call(
            JsBaskupsLog {
              progress: None,
              error: Some(format!("Could not message from {}", log_path.display()).to_string()),
              complete: true,
            },
            napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
          );
          break;
        };
        let log_line = message.to_log();

        tsfn.call(
          JsBaskupsLog {
            progress: Some(log_line),
            error: None,
            complete: false,
          },
          napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
        );
      }

      tsfn.call(
        JsBaskupsLog {
          progress: None,
          error: None,
          complete: true,
        },
        napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
      );
    });

    Ok(AbortHandle::new(handle))
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
}

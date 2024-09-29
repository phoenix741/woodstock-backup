mod abort_handle;
pub mod remove;
pub mod resolve;
pub mod tools;

pub use abort_handle::AbortHandle;

use std::{sync::Arc, time::SystemTime};

use napi::{
  bindgen_prelude::BigInt,
  threadsafe_function::{ErrorStrategy, ThreadsafeFunction},
  Error, JsFunction, Result,
};
use tokio::sync::Mutex;
use woodstock::{
  config::Context,
  server::{
    backup_client::BackupClient, grpc_client::BackupGrpcClient, progression::BackupProgression,
  },
};

use crate::{
  config::context::{JsBackupContext, LogLevel},
  log::{LogBackupContext, LOG_CONTEXT},
};

#[napi(object)]
pub struct WoodstockBackupCommandReply {
  pub code: i32,
  pub stdout: String,
  pub stderr: String,
}

impl From<woodstock::ExecuteCommandReply> for WoodstockBackupCommandReply {
  fn from(reply: woodstock::ExecuteCommandReply) -> Self {
    Self {
      code: reply.code,
      stdout: reply.stdout,
      stderr: reply.stderr,
    }
  }
}

#[napi(object)]
#[derive(Clone)]
pub struct WoodstockBackupShare {
  pub share_path: String,
  pub includes: Vec<String>,
  pub excludes: Vec<String>,
}

impl From<WoodstockBackupShare> for woodstock::Share {
  fn from(share: WoodstockBackupShare) -> Self {
    Self {
      share_path: share.share_path,
      includes: share.includes,
      excludes: share.excludes,
    }
  }
}

#[napi(object)]
pub struct JsBackupProgression {
  pub start_date: i64,
  pub start_transfer_date: Option<i64>,
  pub end_transfer_date: Option<i64>,

  pub compressed_file_size: BigInt,
  pub new_compressed_file_size: BigInt,
  pub modified_compressed_file_size: BigInt,

  pub file_size: BigInt,
  pub new_file_size: BigInt,
  pub modified_file_size: BigInt,

  pub new_file_count: u32,
  pub file_count: u32,
  pub modified_file_count: u32,
  pub removed_file_count: u32,

  pub error_count: u32,

  pub progress_current: BigInt,
  pub progress_max: BigInt,

  pub percent: f64,
  pub speed: f64,
}

#[napi(object)]
pub struct JsBackupProgressionMessage {
  pub progress: Option<JsBackupProgression>,
  pub error: Option<String>,
  pub complete: bool,
}

#[napi(object)]
pub struct JsLogEntry {
  pub level: Option<LogLevel>,
  pub context: Option<String>,
  pub line: Option<String>,
}

impl From<&BackupProgression> for JsBackupProgression {
  fn from(progression: &BackupProgression) -> Self {
    Self {
      start_date: i64::try_from(
        progression
          .start_date
          .duration_since(SystemTime::UNIX_EPOCH)
          .unwrap()
          .as_secs(),
      )
      .unwrap(),
      start_transfer_date: progression.start_transfer_date.map(|date| {
        i64::try_from(
          date
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        )
        .unwrap()
      }),
      end_transfer_date: progression.end_transfer_date.map(|date| {
        i64::try_from(
          date
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        )
        .unwrap()
      }),

      compressed_file_size: BigInt::from(progression.compressed_file_size),
      new_compressed_file_size: BigInt::from(progression.new_compressed_file_size),
      modified_compressed_file_size: BigInt::from(progression.modified_compressed_file_size),

      file_size: BigInt::from(progression.file_size),
      new_file_size: BigInt::from(progression.new_file_size),
      modified_file_size: BigInt::from(progression.modified_file_size),

      new_file_count: progression.new_file_count as u32,
      file_count: progression.file_count as u32,
      modified_file_count: progression.modified_file_count as u32,
      removed_file_count: progression.removed_file_count as u32,

      error_count: progression.error_count as u32,

      progress_current: BigInt::from(progression.progress_current),
      progress_max: BigInt::from(progression.progress_max),

      percent: progression.percent(),
      speed: progression.speed(),
    }
  }
}

#[napi(js_name = "WoodstockBackupClient")]
pub struct WoodstockBackupClient {
  client: Arc<Mutex<BackupClient<BackupGrpcClient>>>,

  ip: String,
  hostname: String,
  backup_number: usize,
}

#[napi]
impl WoodstockBackupClient {
  #[napi(factory)]
  pub async fn create_client(
    hostname: String,
    ip: String,
    backup_number: u32,
    context: &JsBackupContext,
  ) -> Result<Self> {
    let context: Context = context.into();

    let backup_number = usize::try_from(backup_number)
      .map_err(|_| Error::from_reason("Backup number is too large".to_string()))?;

    let grpc_client = BackupGrpcClient::new(&hostname, &ip, &context)
      .await
      .map_err(|_| {
        Error::from_reason(format!("Can't create connection to {hostname} ({ip})").to_string())
      })?;

    let client = BackupClient::new(grpc_client, &hostname, backup_number, &context);

    Ok(Self {
      client: Arc::new(Mutex::new(client)),

      ip,
      hostname,
      backup_number,
    })
  }

  #[napi(getter)]
  pub fn hostname(&self) -> String {
    self.hostname.clone()
  }

  #[napi(getter)]
  pub fn ip(&self) -> String {
    self.ip.clone()
  }

  #[napi(getter)]
  pub fn backup_number(&self) -> u32 {
    u32::try_from(self.backup_number).unwrap()
  }

  #[napi]
  pub async fn authenticate(&self, password: String) -> Result<()> {
    LOG_CONTEXT
      .scope(
        LogBackupContext {
          hostname: self.hostname.clone(),
          backup_number: self.backup_number as u32,
        },
        async {
          let mut client = self.client.lock().await;
          client.authenticate(&password).await.map_err(|_| {
            Error::from_reason("Can't authenticate with the given password".to_string())
          })?;

          Ok(())
        },
      )
      .await
  }

  #[napi]
  pub async fn create_backup_directory(&self, shares: Vec<String>) -> Result<()> {
    LOG_CONTEXT
      .scope(
        LogBackupContext {
          hostname: self.hostname.clone(),
          backup_number: self.backup_number as u32,
        },
        async {
          let client = self.client.lock().await;
          let shares = shares
            .iter()
            .map(std::string::String::as_str)
            .collect::<Vec<_>>();

          client
            .init_backup_directory(&shares)
            .await
            .map_err(|_| Error::from_reason("Can't create backup directory".to_string()))
        },
      )
      .await
  }

  #[napi]
  pub async fn execute_command(&self, command: String) -> Result<WoodstockBackupCommandReply> {
    LOG_CONTEXT
      .scope(
        LogBackupContext {
          hostname: self.hostname.clone(),
          backup_number: self.backup_number as u32,
        },
        async {
          let mut client = self.client.lock().await;
          client
            .execute_command(&command)
            .await
            .map(std::convert::Into::into)
            .map_err(|_| Error::from_reason("Can't execute command".to_string()))
        },
      )
      .await
  }

  #[napi]
  pub fn synchronize_file_list(
    &self,
    share: WoodstockBackupShare,
    #[napi(ts_arg_type = "(result: JsBackupProgressionMessage) => void")] callback: JsFunction,
  ) -> Result<AbortHandle> {
    let tsfn: ThreadsafeFunction<JsBackupProgressionMessage, ErrorStrategy::Fatal> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let share = share.clone();

    let client = self.client.clone();

    let hostname = self.hostname.clone();
    let backup_number = self.backup_number as u32;

    let handle = tokio::spawn(async move {
      LOG_CONTEXT
        .scope(
          LogBackupContext {
            hostname,
            backup_number,
          },
          async {
            let mut client = client.lock().await;
            let result = client
              .synchronize_file_list(&share.into(), &|progression| {
                tsfn.call(
                  JsBackupProgressionMessage {
                    progress: Some(progression.into()),
                    error: None,
                    complete: false,
                  },
                  napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
                );
              })
              .await
              .map_err(|e| Error::from_reason(format!("Can't download file list: {e}")));

            match result {
              Ok(()) => {
                tsfn.call(
                  JsBackupProgressionMessage {
                    progress: None,
                    error: None,
                    complete: true,
                  },
                  napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
                );
              }
              Err(e) => {
                tsfn.call(
                  JsBackupProgressionMessage {
                    progress: None,
                    error: Some(e.to_string()),
                    complete: true,
                  },
                  napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
                );
              }
            }
          },
        )
        .await
    });

    Ok(AbortHandle::new(handle))
  }

  #[napi]
  pub fn create_backup(
    &self,
    share_path: String,
    #[napi(ts_arg_type = "(result: JsBackupProgressionMessage) => void")] callback: JsFunction,
  ) -> Result<AbortHandle> {
    let tsfn: ThreadsafeFunction<JsBackupProgressionMessage, ErrorStrategy::Fatal> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let share_path = share_path.clone();

    let client = self.client.clone();

    let hostname = self.hostname.clone();
    let backup_number = self.backup_number as u32;

    let handle = tokio::spawn(async move {
      LOG_CONTEXT
        .scope(
          LogBackupContext {
            hostname,
            backup_number,
          },
          async {
            let client = client.lock().await;
            let result = client
              .create_backup(&share_path, &|progression| {
                tsfn.call(
                  JsBackupProgressionMessage {
                    progress: Some(progression.into()),
                    error: None,
                    complete: false,
                  },
                  napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
                );
              })
              .await
              .map_err(|e| Error::from_reason(format!("Can't download file list: {e}")));

            match result {
              Ok(()) => {
                tsfn.call(
                  JsBackupProgressionMessage {
                    progress: None,
                    error: None,
                    complete: true,
                  },
                  napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
                );
              }
              Err(e) => {
                tsfn.call(
                  JsBackupProgressionMessage {
                    progress: None,
                    error: Some(e.to_string()),
                    complete: true,
                  },
                  napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
                );
              }
            }
          },
        )
        .await
    });

    Ok(AbortHandle::new(handle))
  }

  #[napi]
  pub async fn compact(&self, share_path: String) -> Result<()> {
    LOG_CONTEXT
      .scope(
        LogBackupContext {
          hostname: self.hostname.clone(),
          backup_number: self.backup_number as u32,
        },
        async {
          let client = self.client.lock().await;
          client
            .compact(&share_path)
            .await
            .map_err(|_| Error::from_reason("Can't compact".to_string()))
        },
      )
      .await
  }

  #[napi]
  pub async fn count_references(&self) -> Result<()> {
    LOG_CONTEXT
      .scope(
        LogBackupContext {
          hostname: self.hostname.clone(),
          backup_number: self.backup_number as u32,
        },
        async {
          let client = self.client.lock().await;
          client
            .count_references()
            .await
            .map_err(|_| Error::from_reason("Can't count references".to_string()))
        },
      )
      .await
  }

  #[napi]
  pub async fn save_backup(&self, finished: bool, completed: bool) -> Result<()> {
    LOG_CONTEXT
      .scope(
        LogBackupContext {
          hostname: self.hostname.clone(),
          backup_number: self.backup_number as u32,
        },
        async {
          let client = self.client.lock().await;
          client
            .save_backup(finished, completed)
            .await
            .map_err(|_| Error::from_reason("Can't save backup".to_string()))
        },
      )
      .await
  }

  #[napi]
  pub async fn close(&self) -> Result<()> {
    LOG_CONTEXT
      .scope(
        LogBackupContext {
          hostname: self.hostname.clone(),
          backup_number: self.backup_number as u32,
        },
        async {
          let mut client = self.client.lock().await;
          client
            .close()
            .await
            .map_err(|_| Error::from_reason("Can't close".to_string()))
        },
      )
      .await
  }
}

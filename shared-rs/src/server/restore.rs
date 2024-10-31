use napi::{
  threadsafe_function::{
    ErrorStrategy::{self},
    ThreadsafeFunction,
  },
  Error, JsFunction, Result,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use woodstock::{
  config::Context,
  server::{
    backup_restore::BackupRestore, grpc_client::BackupGrpcClient, progression::BackupProgression,
  },
};

use crate::{
  config::context::JsBackupContext,
  log::{LogBackupContext, LOG_CONTEXT},
};

use super::{AbortHandle, JsBackupProgressionMessage};

#[napi(js_name = "WoodstockBackupRestore")]
pub struct WoodstockBackupRestore {
  client: Arc<Mutex<BackupRestore<BackupGrpcClient>>>,

  hostname: String,
  backup_number: usize,
}

#[napi]
impl WoodstockBackupRestore {
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
    let client = BackupRestore::new(grpc_client, &hostname, backup_number, &context);

    Ok(Self {
      client: Arc::new(Mutex::new(client)),

      hostname,
      backup_number,
    })
  }

  #[napi(getter)]
  pub fn hostname(&self) -> String {
    self.hostname.clone()
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
  pub async fn prepare_restauration(&self, share: String, selection: Vec<String>) -> Result<()> {
    LOG_CONTEXT
      .scope(
        LogBackupContext {
          hostname: self.hostname.clone(),
          backup_number: self.backup_number as u32,
        },
        async {
          let mut client = self.client.lock().await;

          client
            .prepare_restauration(&share, &selection)
            .await
            .map_err(|_| Error::from_reason("Can't create backup directory".to_string()))
        },
      )
      .await
  }

  #[napi]
  pub fn restore(
    &self,
    share: String,
    destination_directory: String,
    selection: Vec<String>,
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
            let result = {
              let tsfn = tsfn.clone();
              let mut client = client.lock().await;
              client
                .restore(
                  &share,
                  &destination_directory,
                  &selection,
                  Box::new(move |progression: BackupProgression| {
                    tsfn.call(
                      JsBackupProgressionMessage {
                        progress: Some((&progression).into()),
                        error: None,
                        complete: false,
                      },
                      napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
                    );
                  }),
                )
                .await
                .map_err(|e| Error::from_reason(format!("Can't download file list: {e}")))
            };

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

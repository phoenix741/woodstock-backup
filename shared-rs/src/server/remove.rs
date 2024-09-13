use napi::{Error, Result};
use std::sync::Arc;
use tokio::sync::Mutex;
use woodstock::{config::Context, server::backup_remove::BackupRemove};

use crate::{
  config::context::JsBackupContext,
  log::{LogBackupContext, LOG_CONTEXT},
};

#[napi(js_name = "WoodstockBackupRemove")]
pub struct WoodstockBackupRemove {
  client: Arc<Mutex<BackupRemove>>,

  hostname: String,
  backup_number: usize,
}

#[napi]
impl WoodstockBackupRemove {
  #[napi(factory)]
  pub async fn create_client(
    hostname: String,
    backup_number: u32,
    context: &JsBackupContext,
  ) -> Result<Self> {
    let context: Context = context.into();

    let backup_number = usize::try_from(backup_number)
      .map_err(|_| Error::from_reason("Backup number is too large".to_string()))?;

    let client = BackupRemove::new(&hostname, backup_number, &context);

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
  pub async fn remove_backup(&self) -> Result<()> {
    LOG_CONTEXT
      .scope(
        LogBackupContext {
          hostname: self.hostname.clone(),
          backup_number: self.backup_number as u32,
        },
        async {
          let client = self.client.lock().await;
          client
            .remove_backup()
            .await
            .map_err(|_| Error::from_reason("Can't remove backup".to_string()))
        },
      )
      .await
  }

  #[napi]
  pub async fn remove_refcnt_of_host(&self) -> Result<()> {
    LOG_CONTEXT
      .scope(
        LogBackupContext {
          hostname: self.hostname.clone(),
          backup_number: self.backup_number as u32,
        },
        async {
          let client = self.client.lock().await;
          client
            .remove_refcnt_of_host()
            .await
            .map_err(|_| Error::from_reason("Can't remove references count".to_string()))
        },
      )
      .await
  }
}

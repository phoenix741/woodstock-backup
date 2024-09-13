use log::debug;
use napi::{Error, Result};
use woodstock::{config::Context, server::tools::ping};

use crate::config::context::JsBackupContext;

#[napi]
pub async fn grpc_ping(ip: String, hostname: String, context: &JsBackupContext) -> Result<bool> {
  let context: Context = context.into();

  ping(ip, hostname, &context)
    .await
    .map_err(|e| Error::from_reason(e.to_string()))
    .inspect(|&ping| {
      debug!("Ping result: {:?}", ping);
    })
}

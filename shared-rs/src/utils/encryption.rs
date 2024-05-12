use std::path::Path;

use napi::{Error, Result};
use woodstock::{config::Context, utils::encryption::generate_rsa_key as lib_generate_rsa_key};

use crate::config::context::JsBackupContext;

#[napi]
pub fn generate_rsa_key(context: &JsBackupContext) -> Result<()> {
  let context: Context = context.into();
  let certificate_path = Path::new(&context.config.path.certificates_path);

  lib_generate_rsa_key(certificate_path).map_err(|e| Error::from_reason(e.to_string()))
}

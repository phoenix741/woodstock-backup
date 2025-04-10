use napi::{Error, Result};
use woodstock::{
  config::GlobalConfiguration, utils::encryption::generate_rsa_key as lib_generate_rsa_key,
};

#[napi]
pub fn generate_rsa_key() -> Result<()> {
  let certificate_path = &GlobalConfiguration.path.certificates_path;

  lib_generate_rsa_key(certificate_path).map_err(|e| Error::from_reason(e.to_string()))
}

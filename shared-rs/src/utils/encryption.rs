use napi::{Error, Result};
use woodstock::{
  config::GlobalConfiguration, utils::encryption::generate_rsa_key as lib_generate_rsa_key,
};

#[napi]
/// Generates a new RSA key and stores it in the certificates path.
///
/// # Errors
/// Returns an error if the RSA key generation fails or if the key cannot be written to disk.
pub fn generate_rsa_key() -> Result<()> {
  let certificate_path = &GlobalConfiguration.path.certificates_path;

  lib_generate_rsa_key(certificate_path).map_err(|e| Error::from_reason(e.to_string()))
}

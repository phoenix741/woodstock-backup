use dirs::config_dir;
use log::{debug, info};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use eyre::Result;

/// Represents the configuration for the client.
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfig {
    /// The hostname of the client. Defaults to the system's hostname.
    #[serde(default = "ClientConfig::default_hostname")]
    pub hostname: String,
    /// The bind address for the client. Defaults to "0.0.0.0:3657".
    #[serde(default = "ClientConfig::default_bind")]
    pub bind: String,
    /// The password for the client.
    pub password: String,
    /// The secret key for the client. Defaults to a randomly generated 64-byte hexadecimal string.
    #[serde(default = "ClientConfig::default_secret")]
    pub secret: String,
    /// The timeout for backup operations in seconds. Defaults to 3600 seconds (1 hour).
    #[serde(default = "ClientConfig::default_backup_timeout")]
    pub backup_timeout: u64,

    /// If extended attributes should be save on linux platform (default: false)
    #[serde(default)]
    pub xattr: bool,

    /// If the acl should be saved on linux platform
    #[serde(default)]
    pub acl: bool,
}

impl ClientConfig {
    /// Returns the default hostname of the client.
    fn default_hostname() -> String {
        hostname::get()
            .expect("Failed to get hostname")
            .into_string()
            .expect("Failed to convert OsString to String")
    }

    /// Returns the default bind address of the client.
    fn default_bind() -> String {
        "0.0.0.0:3657".to_string()
    }

    /// Generates a random 64-byte hexadecimal string as the default secret key for the client.
    fn default_secret() -> String {
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 64];
        rng.fill_bytes(&mut bytes);
        hex::encode(bytes)
    }

    /// Returns the default backup timeout in seconds.
    fn default_backup_timeout() -> u64 {
        3600
    }
}

impl Default for ClientConfig {
    /// Returns the default configuration for the client.
    fn default() -> Self {
        Self {
            hostname: ClientConfig::default_hostname(),
            bind: ClientConfig::default_bind(),
            password: String::new(),
            secret: ClientConfig::default_secret(),
            backup_timeout: ClientConfig::default_backup_timeout(),
            xattr: false,
            acl: false,
        }
    }
}

/// Reads the client configuration from a file.
///
/// # Arguments
///
/// * `path` - The path to the configuration file.
///
/// # Returns
///
/// Returns a `Result` containing the parsed `ClientConfig` if successful, or an error if the file cannot be read or parsed.
///
/// # Errors
///
/// An error is returned if the file cannot be read or parsed.
///
pub fn read_config<P: AsRef<Path>>(path: P) -> Result<ClientConfig> {
    debug!(
        "Reading client configuration from file: {:?}",
        path.as_ref().display()
    );

    // If the file does not exist, return the default configuration
    if !path.as_ref().exists() {
        return Ok(ClientConfig::default());
    }

    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config: ClientConfig = serde_yaml::from_str(&contents)?;

    info!("Client configuration loaded successfully");

    Ok(config)
}

/// Returns the path to the client configuration file.
///
/// If the environment variable `CLIENT_PATH` is set, it returns the path specified by the variable.
/// Otherwise, if the user is root, it returns the path "/var/lib/woodstock/client".
/// Otherwise, it returns the sub-folder ".woodstock" in the user's configuration directory.
///
/// # Returns
///
/// Returns the path to the client configuration file.
///
/// # Panics
///
/// Panics if the user's configuration directory cannot be determined.
///
#[must_use]
pub fn get_config_path() -> PathBuf {
    debug!("Getting client configuration path");

    // Check if the environment variable CLIENT_PATH is defined
    if let Ok(path) = env::var("CLIENT_PATH") {
        debug!(
            "Using client configuration path from environment variable CLIENT_PATH: {}",
            path
        );

        return PathBuf::from(path);
    }

    // Otherwise, return the ".woodstock" sub-folder in the user's configuration directory
    let mut home = config_dir().expect("Failed to determine user's directory");
    home.push("woodstock");

    debug!("Using client configuration path: {:?}", home);
    home
}
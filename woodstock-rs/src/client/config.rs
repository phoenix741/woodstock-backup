use dirs::config_dir;
use log::{debug, info};
use rand::RngCore;
use serde::Deserialize;
use std::default::Default;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use eyre::Result;

use crate::config::DEFAULT_PORT;

/// Number of seconds in a day, used for the default update check interval.
const DAYLY_UPDATE: u64 = 24 * 3600;

#[derive(Clone, Debug, Deserialize, Default)]
/// Defines the resolution mode used to discover servers.
pub enum ResolutionMode {
    #[cfg(feature = "mdns")]
    /// Uses mDNS (multicast DNS) for server discovery.
    Mdns,
    #[default]
    /// Uses direct DNS resolution for server discovery.
    Direct,
    /// Disables automatic server discovery.
    None,
}

/// Represents the configuration for the client.
#[derive(Clone, Debug, Deserialize)]
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
    /// The max duration of a backup operation in seconds. Defaults to 43200 seconds (12 hours).
    #[serde(default = "ClientConfig::default_max_backup_seconds")]
    pub max_backup_seconds: u64,

    #[serde(default)]
    /// The resolution mode for server discovery. Defaults to the value specified by `ResolutionMode::default()`.
    pub resolution_mode: ResolutionMode,

    #[serde(default)]
    /// Optional list of network interfaces to use for mDNS resolution.
    pub mdns_interfaces: Option<Vec<String>>,

    #[serde(default)]
    /// Optional server hostname or address to connect to directly.
    pub server: Option<String>,

    /// If the restauration should be disabled, for security reason
    #[serde(default)]
    pub disable_restauration: bool,

    /// If extended attributes should be save on linux platform (default: false)
    #[serde(default)]
    pub xattr: bool,

    /// If the acl should be saved on linux platform
    #[serde(default)]
    pub acl: bool,

    /// Update automatically client daemon
    #[serde(default = "ClientConfig::default_automatic_update")]
    pub auto_update: bool,

    /// Delay between two update check
    #[serde(default = "ClientConfig::default_update_delay")]
    pub update_delay: u64,

    /// Log directory
    #[serde(default)]
    pub log_directory: Option<PathBuf>,
}

impl ClientConfig {
    /// Returns the default hostname of the client.
    ///
    /// # Panics
    ///
    /// Panics if the hostname cannot be retrieved from the system or if the hostname
    /// cannot be converted to a string.
    fn default_hostname() -> String {
        hostname::get()
            .expect("Failed to get hostname")
            .into_string()
            .expect("Failed to convert OsString to String")
    }

    /// Returns the default bind address of the client.
    fn default_bind() -> String {
        format!("0.0.0.0:{DEFAULT_PORT}").to_string()
    }

    /// Generates a random 64-byte hexadecimal string as the default secret key for the client.
    fn default_secret() -> String {
        let mut rng = rand::rng();
        let mut bytes = [0u8; 64];
        rng.fill_bytes(&mut bytes);
        hex::encode(bytes)
    }

    /// Returns the default backup timeout in seconds.
    /// Defaults to 1 hours.
    fn default_backup_timeout() -> u64 {
        3600
    }

    /// Returns the default max backup duration in seconds.
    /// Defaults to 12 hours.
    fn default_max_backup_seconds() -> u64 {
        12 * 3600
    }

    #[must_use]
    /// Returns the current version of the client package.
    ///
    /// # Returns
    ///
    /// The version string from Cargo package metadata.
    pub fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    #[must_use]
    /// Returns the default value for automatic updates.
    ///
    /// # Returns
    ///
    /// Returns `true` on Windows, `false` on other platforms.
    pub fn default_automatic_update() -> bool {
        cfg!(windows)
    }

    #[must_use]
    /// Returns the default delay between update checks.
    ///
    /// # Returns
    ///
    /// The delay in seconds (defaults to one day).
    pub fn default_update_delay() -> u64 {
        DAYLY_UPDATE
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
            max_backup_seconds: ClientConfig::default_max_backup_seconds(),
            resolution_mode: ResolutionMode::default(),
            disable_restauration: false,
            mdns_interfaces: None,
            server: None,
            xattr: false,
            acl: false,
            auto_update: ClientConfig::default_automatic_update(),
            update_delay: ClientConfig::default_update_delay(),
            log_directory: None,
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

use log::{debug, info};
use serde::Deserialize;
use std::path::Path;
use tokio::fs::read_to_string;
use woodstock::config::HostConfiguration;

use eyre::Result;

/// Represents the configuration for the client.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StandaloneClientConfig {
    /// If extended attributes should be save on linux platform (default: false)
    #[serde(default)]
    pub xattr: bool,

    /// If the acl should be saved on linux platform
    #[serde(default)]
    pub acl: bool,

    pub backup_configuration: HostConfiguration,
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
pub async fn read_standalone_config<P: AsRef<Path>>(path: P) -> Result<StandaloneClientConfig> {
    debug!(
        "Reading standalone client configuration from file: {:?}",
        path.as_ref().display()
    );

    // If the file does not exist, return the default configuration
    if !path.as_ref().exists() {
        return Err(eyre::eyre!(
            "Configuration file does not exist: {}",
            path.as_ref().display()
        ));
    }

    let contents = read_to_string(path.as_ref()).await?;
    let config: StandaloneClientConfig = serde_yaml::from_str(&contents)?;

    info!("Standalone client configuration loaded successfully");

    Ok(config)
}

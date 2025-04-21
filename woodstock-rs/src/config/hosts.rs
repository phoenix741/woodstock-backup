use std::path::{Path, PathBuf};

use eyre::{eyre, Result};
use log::{debug, warn};
use tokio::fs::read_to_string;

use super::{Configuration, HostConfiguration};

/// # Hosts Configuration Module
///
/// This module provides the [`Hosts`] struct and associated methods for managing host configuration
/// files in the Woodstock backup system. It is responsible for listing available hosts, loading
/// host-specific configuration, and reading host configuration files from disk.
///
/// ## Main Structure
///
/// - [`Hosts`]: Central struct for managing host configuration files.
///
/// ## Key Methods
///
/// - [`Hosts::new`]: Create a new `Hosts` manager from a configuration.
/// - [`Hosts::list_hosts`]: List all hostnames known to the system.
/// - [`Hosts::get_host`]: Load the configuration for a specific host.
/// - [`Hosts::read_host_file`]: Read and parse a host configuration file.
///
/// ## Error Handling
///
/// - Most methods return `Result` and propagate I/O or deserialization errors using the `eyre` crate.
/// - If a host is not found, an error is returned.
///
/// ## Panics
///
/// - Methods do not panic under normal operation. Errors are returned as `Result`.
///
/// ## Thread Safety
///
/// This struct is not thread-safe by itself. If used in a concurrent context, wrap in a mutex or use only from one thread.
///
/// ## See Also
///
/// - [`HostConfiguration`]: For host-specific settings
pub struct Hosts {
    /// Path to the hosts configuration file (hosts.yml).
    config_path_hosts: PathBuf,
    /// Path to the directory containing host configuration files.
    config_path: PathBuf,
}

impl Hosts {
    /// Creates a new `Hosts` manager from the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Reference to the Woodstock [`Configuration`] struct.
    ///
    /// # Returns
    ///
    /// A new instance of [`Hosts`] with paths initialized from the configuration.
    #[must_use]
    pub fn new(config: &Configuration) -> Self {
        Self {
            config_path_hosts: config.path.config_path_hosts.clone(),
            config_path: config.path.config_path.clone(),
        }
    }

    /// Lists all hostnames known to the system.
    ///
    /// Reads the `hosts.yml` file and returns a vector of hostnames.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - List of hostnames if the file is readable and valid.
    /// * `Err(eyre::Report)` - If the file cannot be read or parsed (returns empty list on error).
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be parsed as YAML, but not if the file is missing.
    pub async fn list_hosts(&self) -> Result<Vec<String>> {
        debug!("Reading hosts from {:?}", self.config_path_hosts);

        let hosts = read_to_string(&self.config_path_hosts).await;
        let hosts = match hosts {
            Ok(hosts) => {
                debug!("Hosts file content: {hosts}");
                let hosts: Vec<String> = serde_yaml::from_str(&hosts)?;
                hosts
            }
            Err(e) => {
                warn!("Error reading hosts file: {e}");
                vec![]
            }
        };

        Ok(hosts)
    }

    /// Loads the configuration for a specific host.
    ///
    /// Checks if the host exists in the list, then loads its configuration file.
    ///
    /// # Arguments
    ///
    /// * `hostname` - The name of the host to load.
    ///
    /// # Returns
    ///
    /// * `Ok(HostConfiguration)` - The configuration for the host if found and valid.
    /// * `Err(eyre::Report)` - If the host is not found or the file cannot be read/parsed.
    ///
    /// # Errors
    ///
    /// Returns an error if the host is not listed or the configuration file is invalid.
    pub async fn get_host(&self, hostname: &str) -> Result<HostConfiguration> {
        // Check if the host is in the list
        let hosts = self.list_hosts().await?;
        if !hosts.contains(&hostname.to_string()) {
            return Err(eyre!("Host {hostname} not found"));
        }

        let path = self.get_host_configuration_file(hostname);
        let host = self.read_host_file(path).await?;

        Ok(host)
    }

    /// Reads and parses a host configuration file from disk.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the host configuration file (can be any type implementing `AsRef<Path>`).
    ///
    /// # Returns
    ///
    /// * `Ok(HostConfiguration)` - The parsed host configuration.
    /// * `Err(eyre::Report)` - If the file cannot be read or parsed as YAML.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub async fn read_host_file<P: AsRef<Path>>(&self, path: P) -> Result<HostConfiguration> {
        let content = read_to_string(path).await?;
        let host: HostConfiguration = serde_yaml::from_str(&content)?;

        Ok(host)
    }

    /// Returns the path to the configuration file for a given host.
    ///
    /// # Arguments
    ///
    /// * `hostname` - The name of the host.
    ///
    /// # Returns
    ///
    /// The path to the YAML configuration file for the specified host.
    fn get_host_configuration_file(&self, hostname: &str) -> PathBuf {
        self.config_path.join(format!("{hostname}.yml"))
    }
}

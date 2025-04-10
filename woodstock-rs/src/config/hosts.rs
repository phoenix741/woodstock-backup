use std::path::{Path, PathBuf};

use eyre::{eyre, Result};
use log::{debug, warn};
use tokio::fs::read_to_string;

use super::{Configuration, HostConfiguration};

pub struct Hosts {
    config_path_hosts: PathBuf,
    config_path: PathBuf,
}

impl Hosts {
    #[must_use]
    pub fn new(config: &Configuration) -> Self {
        Self {
            config_path_hosts: config.path.config_path_hosts.clone(),
            config_path: config.path.config_path.clone(),
        }
    }

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

    pub async fn read_host_file<P: AsRef<Path>>(&self, path: P) -> Result<HostConfiguration> {
        let content = read_to_string(path).await?;
        let host: HostConfiguration = serde_yaml::from_str(&content)?;

        Ok(host)
    }

    fn get_host_configuration_file(&self, hostname: &str) -> PathBuf {
        self.config_path.join(format!("{hostname}.yml"))
    }
}

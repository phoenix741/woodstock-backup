use std::path::PathBuf;

use eyre::{eyre, Result};
use log::debug;
use tokio::fs::read_to_string;

use super::{Context, HostConfiguration};

pub struct Hosts {
    config_path_hosts: PathBuf,
    config_path: PathBuf,
}

impl Hosts {
    #[must_use]
    pub fn new(ctxt: &Context) -> Self {
        Self {
            config_path_hosts: ctxt.config.path.config_path_hosts.clone(),
            config_path: ctxt.config.path.config_path.clone(),
        }
    }

    pub async fn list_hosts(&self) -> Result<Vec<String>> {
        debug!("Reading hosts from {:?}", self.config_path_hosts);

        let hosts = read_to_string(&self.config_path_hosts).await?;
        let hosts: Vec<String> = serde_yaml::from_str(&hosts)?;
        Ok(hosts)
    }

    pub async fn get_host(&self, hostname: &str) -> Result<HostConfiguration> {
        // Check if the host is in the list
        let hosts = self.list_hosts().await?;
        if !hosts.contains(&hostname.to_string()) {
            return Err(eyre!("Host {hostname} not found"));
        }

        let path = self.get_host_configuration_file(hostname);
        let content = read_to_string(path).await?;
        let host: HostConfiguration = serde_yaml::from_str(&content)?;

        Ok(host)
    }

    fn get_host_configuration_file(&self, hostname: &str) -> PathBuf {
        self.config_path.join(format!("{hostname}.yml"))
    }
}

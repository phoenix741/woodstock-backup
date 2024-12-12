use std::{sync::Arc, thread, time::Duration};

use eyre::Result;
use if_addrs::IfAddr;
use log::error;
use reqwest::Client;
use serde::Serialize;
use serde_json::json;
use tokio::{sync::Mutex, task::AbortHandle};

use super::ResolveClient;
use crate::client::config::ClientConfig;

#[derive(Serialize)]
struct Ipv4Addr {
    addr: String,
    netmask: String,
}

#[derive(Serialize)]
struct Ipv6Addr {
    addr: String,
    netmask: String,
}

#[derive(Serialize)]
struct InterfaceInfo {
    name: String,
    ipv4: Option<Ipv4Addr>,
    ipv6: Option<Ipv6Addr>,
}

#[derive(Clone)]
pub struct DirectResolveClient {
    client: Client,
    uri: String,
    config: ClientConfig,
    refresher: Arc<Mutex<Option<tokio::task::AbortHandle>>>,
}

const REFRESH_INTERVAL: u64 = 60;

impl DirectResolveClient {
    pub async fn new(config: ClientConfig) -> Result<Self> {
        let config_clone = config.clone();
        let client = Client::new();
        let uri = format!(
            "https://{}/api/hosts/{}/client",
            config.server.unwrap_or_default(),
            config.hostname
        );
        let daemon = Self {
            client,
            uri,
            config: config_clone,
            refresher: Arc::new(Mutex::new(None)),
        };

        Ok(daemon)
    }

    async fn refresh(&self) -> Result<()> {
        let interfaces = self.list_interfaces(&self.config)?;

        let response = self
            .client
            .post(&self.uri)
            .json(&json!({
                "addresses": interfaces,
                "version": ClientConfig::version(),
            }))
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            error!("Failed to register address: {}", status);
        }

        Ok(())
    }

    fn list_interfaces(&self, config: &ClientConfig) -> Result<Vec<InterfaceInfo>> {
        let config_mdns_interfaces = config.mdns_interfaces.clone();
        let config_mdns_interfaces = config_mdns_interfaces.as_ref();

        let interfaces = if_addrs::get_if_addrs()?;
        let interfaces = interfaces
            .iter()
            .filter(|iface| {
                let name = &iface.name;
                config_mdns_interfaces
                    .map(|f| f.iter().any(|iface_name| iface_name == name))
                    .unwrap_or(true)
            })
            .map(|iface| {
                let ipv4 = if let IfAddr::V4(addr) = &iface.addr {
                    Some(Ipv4Addr {
                        addr: addr.ip.to_string(),
                        netmask: addr.netmask.to_string(),
                    })
                } else {
                    None
                };
                let ipv6 = if let IfAddr::V6(addr) = &iface.addr {
                    Some(Ipv6Addr {
                        addr: addr.ip.to_string(),
                        netmask: addr.netmask.to_string(),
                    })
                } else {
                    None
                };

                InterfaceInfo {
                    name: iface.name.clone(),
                    ipv4,
                    ipv6,
                }
            })
            .collect();

        Ok(interfaces)
    }

    fn thread_refresher(&self) -> AbortHandle {
        let self_clone = self.clone();
        let handler = tokio::spawn(async move {
            loop {
                thread::sleep(Duration::from_secs(REFRESH_INTERVAL));
                if let Err(err) = self_clone.refresh().await {
                    error!("Failed to refresh direct service: {}", err);
                }
            }
        });

        handler.abort_handle()
    }
}

#[tonic::async_trait]
impl ResolveClient for DirectResolveClient {
    async fn start(&self) -> Result<()> {
        self.stop().await;

        let observer = self.thread_refresher();
        self.refresher.lock().await.replace(observer);

        Ok(())
    }

    async fn stop(&self) {
        self.shutdown().await;
    }

    async fn shutdown(&self) {
        if let Some(observer) = self.refresher.lock().await.take() {
            observer.abort();
        }
    }
}

use eyre::Result;
use log::{error, info};
use mdns_sd::{IfKind, ServiceDaemon, ServiceInfo};
use tokio::sync::Mutex;
use tokio::task::AbortHandle;

use crate::client::config::ClientConfig;
use crate::config::{MDNS_SERVICE_NAME, MDNS_SUFFIX};

use super::ResolveClient;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

const CHECK_INTERVAL: u64 = 10;
const MAX_INTERVAL: u64 = CHECK_INTERVAL * 6;

fn create_service_info(config: &ClientConfig) -> Result<ServiceInfo> {
    let addr: std::net::SocketAddr = config.bind.parse()?;
    let port = addr.port();

    let properties = [("version", ClientConfig::version())];

    let service_info = ServiceInfo::new(
        MDNS_SERVICE_NAME,
        &config.hostname,
        &format!("{}{}", &config.hostname, MDNS_SUFFIX),
        "",
        port,
        &properties[..],
    )?;

    let service_info = service_info.enable_addr_auto();

    Ok(service_info)
}

#[derive(Clone)]
pub struct MdnsResolveClient {
    daemon: Arc<Mutex<Option<ServiceDaemon>>>,
    observer: Arc<Mutex<Option<tokio::task::AbortHandle>>>,
    config: ClientConfig,
}

impl MdnsResolveClient {
    pub async fn new(config: ClientConfig) -> Result<Self> {
        let daemon = Self {
            daemon: Arc::new(Mutex::new(None)),
            observer: Arc::new(Mutex::new(None)),
            config,
        };
        daemon.start().await?;

        let observer = daemon.wakeup_observe();
        daemon.observer.lock().await.replace(observer);

        Ok(daemon)
    }

    fn full_name(&self) -> String {
        format!("{}{}", &self.config.hostname, MDNS_SUFFIX)
    }

    async fn refresh(&self) {
        // Start by stopping the current daemon
        self.stop().await;

        // Sleep 10 seconds to let the daemon unregister the service
        tokio::time::sleep(Duration::from_secs(10)).await;

        if let Err(e) = self.start().await {
            error!("Failed to refresh mDNS service: {}", e);
        }
    }

    fn wakeup_observe(&self) -> AbortHandle {
        let self_clone = self.clone();
        let handler = tokio::spawn(async move {
            let mut current_time = std::time::Instant::now();
            loop {
                thread::sleep(Duration::from_secs(CHECK_INTERVAL));
                if std::time::Instant::elapsed(&current_time).as_secs() > MAX_INTERVAL {
                    info!("Device woke up, refreshing mDNS service");
                    self_clone.refresh().await;
                }
                current_time = std::time::Instant::now();
            }
        });

        handler.abort_handle()
    }

    fn list_interfaces(&self, config: &ClientConfig) -> Vec<IfKind> {
        config
            .mdns_interfaces
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|s| IfKind::Name(s.clone()))
            .collect()
    }
}

#[tonic::async_trait]
impl ResolveClient for MdnsResolveClient {
    async fn start(&self) -> Result<()> {
        let mdns = ServiceDaemon::new()?;
        let my_service = create_service_info(&self.config)?;

        // Start by checking all interfaces that match network
        if let Some(network) = &self.config.mdns_interfaces {
            if !network.is_empty() {
                let interfaces = self.list_interfaces(&self.config);
                mdns.disable_interface(IfKind::All)?;
                for interface in interfaces {
                    mdns.enable_interface(interface)?;
                }
            }
        }

        // Register with the daemon, which publishes the service.
        mdns.register(my_service)?;

        info!("Service mDNS enregistré et disponible.");

        self.daemon.lock().await.replace(mdns);

        Ok(())
    }

    async fn stop(&self) {
        if let Some(daemon) = self.daemon.lock().await.take() {
            match daemon.unregister(&self.full_name()) {
                Ok(receiver) => match receiver.recv() {
                    Ok(status) => info!("Service mDNS retiré avec succès: {:?}.", status),
                    Err(e) => error!("Échec de la suppression du service mDNS: {}", e),
                },
                Err(e) => error!("Failed to unregister mDNS service: {}", e),
            }

            match daemon.shutdown() {
                Ok(receiver) => match receiver.recv() {
                    Ok(status) => info!("Service mDNS daemon arrêté avec succès: {:?}.", status),
                    Err(e) => error!("Échec de l'arrêt du service mDNS daemon: {}", e),
                },
                Err(e) => error!("Échec de l'arrêt du service mDNS daemon: {}", e),
            }
        }
    }

    async fn shutdown(&self) {
        if let Some(observer) = self.observer.lock().await.take() {
            observer.abort();
        }

        self.stop().await;
    }
}

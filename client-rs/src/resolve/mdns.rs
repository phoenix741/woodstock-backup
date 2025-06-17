use eyre::Result;
use log::{error, info};
use mdns_sd::{IfKind, ServiceDaemon, ServiceInfo};
use tokio::sync::Mutex;
use tokio::task::AbortHandle;
use woodstock::config::{MDNS_SERVICE_NAME, MDNS_SUFFIX};

use crate::config::ClientConfig;

use super::ResolveClient;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Interval in seconds between system state checks.
const CHECK_INTERVAL: u64 = 10;

/// Maximum interval in seconds before considering the system has woken up
/// and requires an mDNS service reset.
const MAX_INTERVAL: u64 = CHECK_INTERVAL * 6;

/// Creates an mDNS service information record from client configuration.
///
/// This function generates a `ServiceInfo` for the mDNS protocol that advertises
/// the availability of this client on the local network.
///
/// # Arguments
/// * `config` - Client configuration containing necessary information
///   such as binding address and hostname.
///
/// # Returns
/// * `Result<ServiceInfo>` - A result containing the service information if successful.
///
/// # Errors
/// Returns an error if:
/// * The binding address in the configuration cannot be parsed
/// * The service information creation fails
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

/// Implementation client for server resolution via mDNS.
///
/// This client registers an mDNS service on the local network to announce
/// its presence to Woodstock servers, allowing servers to discover
/// clients automatically without manual IP address configuration.
///
/// It also monitors the system state to reactivate the service
/// after a system sleep.
#[derive(Clone)]
pub struct MdnsResolveClient {
    /// mDNS service daemon that manages service announcement on the network.
    daemon: Arc<Mutex<Option<ServiceDaemon>>>,
    /// Handler for the background task that monitors system wake-ups.
    observer: Arc<Mutex<Option<tokio::task::AbortHandle>>>,
    /// Client configuration.
    config: ClientConfig,
}

impl MdnsResolveClient {
    /// Creates a new instance of `MdnsResolveClient`.
    ///
    /// Initializes an mDNS client with the provided configuration and immediately starts
    /// the service announcement process and system wake-up monitoring.
    ///
    /// # Arguments
    /// * `config` - Client configuration containing the necessary information
    ///   for mDNS service announcement.
    ///
    /// # Returns
    /// * `Result<Self>` - A result containing the new `MdnsResolveClient` instance.
    ///
    /// # Errors
    /// Returns an error if:
    /// * The mDNS daemon cannot be started
    /// * The mDNS service registration fails
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

    /// Generates the complete service name for mDNS.
    ///
    /// Combines the client hostname with the mDNS suffix to create
    /// a unique and complete service name.
    ///
    /// # Returns
    /// * `String` - The complete mDNS service name for this client.
    fn full_name(&self) -> String {
        format!("{}{}", &self.config.hostname, MDNS_SUFFIX)
    }

    /// Refreshes the mDNS service by completely restarting it.
    ///
    /// This method is primarily used after a system wake-up
    /// to ensure that the mDNS service is properly re-registered
    /// on the network.
    ///
    /// # Behavior
    /// This method first stops the existing service, waits 10 seconds
    /// to allow for complete unregistration, then restarts the service.
    /// Errors are logged but not propagated.
    async fn refresh(&self) {
        // Start by stopping the current daemon
        self.stop().await;

        // Sleep 10 seconds to let the daemon unregister the service
        tokio::time::sleep(Duration::from_secs(10)).await;

        if let Err(e) = self.start().await {
            error!("Failed to refresh mDNS service: {}", e);
        }
    }

    /// Creates a background task that monitors system wake-up events.
    ///
    /// This method spawns a tokio task that runs continuously, checking
    /// for elapsed time gaps that might indicate the system was in sleep mode.
    /// When such a gap is detected, the mDNS service is automatically refreshed.
    ///
    /// # Returns
    /// * `AbortHandle` - Handle to abort the monitoring task if needed.
    ///
    /// # Behavior
    /// The task runs indefinitely in the background until aborted. It periodically
    /// checks the elapsed time since the last check, and if it exceeds `MAX_INTERVAL`,
    /// it triggers a service refresh.
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

    /// Converts client interface configuration to mDNS interface kinds.
    ///
    /// Takes the interface names from the client configuration and converts them
    /// to the appropriate `IfKind` types used by the mDNS service daemon.
    ///
    /// # Arguments
    /// * `config` - Client configuration containing interface names to use.
    ///
    /// # Returns
    /// * `Vec<IfKind>` - List of interface kinds to enable for mDNS.
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
    /// Starts the mDNS service for this client.
    ///
    /// This method initializes the mDNS service daemon, configures the network interfaces
    /// to use, and registers the service for discovery by Woodstock servers.
    ///
    /// # Returns
    /// * `Result<()>` - A result indicating success or failure of the service start.
    ///
    /// # Errors
    /// Returns an error if:
    /// * The mDNS daemon cannot be created
    /// * Interface configuration fails
    /// * Service registration fails
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

        info!("mDNS service registered and available.");

        self.daemon.lock().await.replace(mdns);

        Ok(())
    }

    /// Stops the mDNS service for this client.
    ///
    /// This method unregisters the service from the mDNS daemon and shuts down
    /// the daemon itself, properly cleaning up resources.
    ///
    /// # Behavior
    /// If no daemon is currently running, this method has no effect.
    /// Any errors during unregistration or shutdown are logged but do not
    /// prevent the method from completing.
    async fn stop(&self) {
        if let Some(daemon) = self.daemon.lock().await.take() {
            match daemon.unregister(&self.full_name()) {
                Ok(receiver) => match receiver.recv() {
                    Ok(status) => info!("mDNS service successfully unregistered: {:?}.", status),
                    Err(e) => error!("Failed to unregister mDNS service: {}", e),
                },
                Err(e) => error!("Failed to unregister mDNS service: {}", e),
            }

            match daemon.shutdown() {
                Ok(receiver) => match receiver.recv() {
                    Ok(status) => info!("mDNS daemon successfully stopped: {:?}.", status),
                    Err(e) => error!("Failed to stop mDNS daemon: {}", e),
                },
                Err(e) => error!("Failed to stop mDNS daemon: {}", e),
            }
        }
    }

    /// Completely shuts down the mDNS client.
    ///
    /// This method terminates the wake-up monitoring task and stops the mDNS service.
    /// It should be called when the client is being fully shut down to ensure proper
    /// cleanup of all resources.
    ///
    /// # Behavior
    /// First aborts any active wake-up monitoring task, then stops the mDNS service.
    async fn shutdown(&self) {
        if let Some(observer) = self.observer.lock().await.take() {
            observer.abort();
        }

        self.stop().await;
    }
}

use std::{sync::Arc, time::Duration};

use eyre::Result;
use if_addrs::IfAddr;
use reqwest::{Certificate, Client, Identity};
use serde::Serialize;
use serde_json::json;
use tokio::{sync::Mutex, task::AbortHandle};
use tracing::{debug, error};

use super::ResolveClient;
use crate::config::ClientConfig;

/// Represents an IPv4 address with its netmask.
#[derive(Serialize, Debug)]
struct Ipv4Addr {
    /// The IPv4 address as a string.
    addr: String,
    /// The netmask as a string.
    netmask: String,
}

/// Represents an IPv6 address with its netmask.
#[derive(Serialize, Debug)]
struct Ipv6Addr {
    /// The IPv6 address as a string.
    addr: String,
    /// The netmask as a string.
    netmask: String,
}

#[derive(Serialize, Debug)]
/// Represents information about a network interface.
struct InterfaceInfo {
    /// The name of the interface.
    name: String,
    /// The IPv4 address information, if available.
    ipv4: Option<Ipv4Addr>,
    /// The IPv6 address information, if available.
    ipv6: Option<Ipv6Addr>,
}

#[derive(Clone)]
/// Implementation for direct resolution of available client.
///
/// This client registers the client's address on the server using direct HTTP calls
/// and periodically refreshes this registration.
///
/// If the server doesn't receive a refresh in a period of two minutes, the server will
/// consider that the client is offline.
///
/// For this implementations work, the client MUST known the server IP and the client API port.
/// The client authentificate to the server with help of client certificate.
pub struct DirectResolveClient {
    /// The URI endpoint of the server for registration.
    uri: String,
    /// Client configuration.
    config: ClientConfig,
    /// The port the client is listening on.
    port: u16,
    /// Handle for the background task that refreshes the registration.
    refresher: Arc<Mutex<Option<tokio::task::AbortHandle>>>,
    /// The root certificate for TLS verification.
    root_ca: Certificate,
    /// The identity (certificate + private key) for client authentication for TLS verification.
    identity: Identity,
}

/// Interval in seconds between registration refresh operations.
const REFRESH_INTERVAL: u64 = 60;

impl DirectResolveClient {
    /// Creates a new `DirectResolveClient` instance.
    ///
    /// This method initializes a new resolver with the provided configuration and security
    /// credentials, and immediately starts the registration process with the server.
    ///
    /// # Arguments
    /// * `config` - The client configuration.
    /// * `identity` - The TLS identity (certificate and private key) for client authentication.
    /// * `root_ca` - The root certificate for verifying the server's TLS certificate.
    ///
    /// # Returns
    /// A Result containing the new `DirectResolveClient` instance.
    ///
    /// # Errors
    /// Returns an error if:
    /// * The bind address in the config cannot be parsed
    /// * The client couldn't be started
    pub async fn new(
        config: ClientConfig,
        identity: Identity,
        root_ca: Certificate,
    ) -> Result<Self> {
        let config_clone = config.clone();

        let uri = format!(
            "{}/api/hosts/{}/client",
            config.server.unwrap_or_default(),
            config.hostname
        );

        let addr: std::net::SocketAddr = config.bind.parse()?;
        let port = addr.port();

        let daemon = Self {
            uri,
            port,
            config: config_clone,
            refresher: Arc::new(Mutex::new(None)),
            root_ca,
            identity,
        };
        daemon.start().await?;

        Ok(daemon)
    }

    /// Creates an HTTP client with TLS configuration.
    ///
    /// Builds a reqwest Client with appropriate TLS settings for secure communication
    /// with the server.
    ///
    /// The resolver trusts only the Woodstock CA provided in configuration.
    ///
    /// Hostname verification is explicitly relaxed because the server can be
    /// reached through its IP address instead of a DNS name. With reqwest 0.13
    /// this requires `tls_certs_only()` instead of `add_root_certificate()`.
    ///
    /// # Returns
    /// A Result containing the configured HTTP client.
    ///
    /// # Errors
    /// Returns an error if the client builder fails to construct the client.
    fn create_client(&self) -> Result<Client> {
        let root_ca = self.root_ca.clone();
        let client = Client::builder()
            .use_rustls_tls()
            .tls_certs_only([root_ca])
            .tls_danger_accept_invalid_hostnames(true)
            .identity(self.identity.clone())
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(client)
    }

    /// Refreshes the client registration with the server.
    ///
    /// Sends a POST request to the server with information about this client's
    /// network interfaces, listening port, and version.
    ///
    /// # Returns
    /// A Result indicating whether the registration was successful.
    ///
    /// # Errors
    /// Returns an error if:
    /// * Creating the HTTP client fails
    /// * Getting interface information fails
    /// * The HTTP request fails
    async fn refresh(&self) -> Result<()> {
        let client = self.create_client()?;
        let interfaces = Self::list_interfaces(&self.config)?;

        let response = client
            .post(&self.uri)
            .json(&json!({
                "addresses": interfaces,
                "port": self.port,
                "version": ClientConfig::version(),
            }))
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            return Err(eyre::eyre!("Failed to register address: {}", status));
        }

        Ok(())
    }

    /// Lists all network interfaces based on the configuration.
    ///
    /// Collects information about network interfaces, filtered according to
    /// the configuration's `mdns_interfaces` setting.
    ///
    /// # Arguments
    /// * `config` - The client configuration containing interface filtering information.
    ///
    /// # Returns
    /// A Result containing a vector of `InterfaceInfo` structures.
    ///
    /// # Errors
    /// Returns an error if getting the system's interface addresses fails.
    fn list_interfaces(config: &ClientConfig) -> Result<Vec<InterfaceInfo>> {
        let config_mdns_interfaces = config.mdns_interfaces.clone();
        let config_mdns_interfaces = config_mdns_interfaces.as_ref();

        let interfaces = if_addrs::get_if_addrs()?;
        let interfaces = interfaces
            .iter()
            .filter(|iface| {
                let name = &iface.name;
                config_mdns_interfaces.is_none_or(|f| f.iter().any(|iface_name| iface_name == name))
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

    /// Creates a background task that periodically refreshes the registration.
    ///
    /// Spawns a tokio task that runs indefinitely, refreshing the client's
    /// registration with the server at regular intervals defined by `REFRESH_INTERVAL`.
    ///
    /// # Returns
    /// An `AbortHandle` that can be used to stop the background task.
    fn thread_refresher(&self) -> AbortHandle {
        let self_clone = self.clone();
        let handler = tokio::spawn(async move {
            loop {
                debug!("Refreshing direct service");
                if let Err(err) = self_clone.refresh().await {
                    error!("Failed to refresh direct service: {:?}", err);
                }
                // Use async sleep to avoid blocking a Tokio worker
                tokio::time::sleep(Duration::from_secs(REFRESH_INTERVAL)).await;
            }
        });

        handler.abort_handle()
    }
}

#[tonic::async_trait]
impl ResolveClient for DirectResolveClient {
    /// Starts the direct resolution client by initializing the periodic registration process.
    ///
    /// This method first stops any existing registration task, then initializes
    /// a new one that periodically sends this client's information to the server.
    ///
    /// # Returns
    /// * `Result<()>` - A result indicating whether the startup was successful.
    ///
    /// # Errors
    /// Returns an error if the registration task creation fails.
    async fn start(&self) -> Result<()> {
        self.stop().await;

        let observer = self.thread_refresher();
        self.refresher.lock().await.replace(observer);

        Ok(())
    }

    /// Stops the direct resolution client.
    ///
    /// This method terminates the periodic registration process by calling
    /// the `shutdown` method.
    ///
    /// # Behavior
    /// Calling this method does not generate an error, even if no process
    /// was running.
    async fn stop(&self) {
        self.shutdown().await;
    }

    /// Cleans up resources used by the client.
    ///
    /// This method terminates the background task that performs periodic registration
    /// by aborting it via its AbortHandle.
    ///
    /// # Behavior
    /// If no registration task is currently running, this method
    /// does nothing.
    async fn shutdown(&self) {
        if let Some(observer) = self.refresher.lock().await.take() {
            observer.abort();
        }
    }
}

use crate::config::DNS_RESOLVE_TIMEOUT_SEC;
#[cfg(all(feature = "mdns", not(target_os = "freebsd")))]
use crate::config::{MDNS_SERVICE_NAME, MDNS_SUFFIX, MDNS_TIMEOUT_MSEC};
#[cfg(all(feature = "mdns", not(target_os = "freebsd")))]
use mdns_sd::{HostnameResolutionEvent, ResolvedService, ScopedIp, ServiceDaemon, ServiceEvent};

use dns_lookup::lookup_host;
use eyre::Result;
use redis::{
    from_redis_value, AsyncCommands, FromRedisValue, RedisError, RedisResult, ToRedisArgs, Value,
};
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use tokio::{net::TcpStream, task::spawn_blocking, time::timeout};
use tracing::{debug, info};

use crate::config::REDIS_WOODSTOCK_KEY_DNS;

/// The interval in seconds for direct DNS update checks.
const DIRECT_DNS_UPDATE_INTERVAL: i64 = 120;

/// Checks if a specific IP address and port are reachable.
///
/// # Arguments
/// * `ip` - The IP address to check.
/// * `port` - The port to check.
///
/// # Returns
///
/// * `bool` - `true` if the address is reachable, `false` otherwise.
async fn is_reachable(ip: IpAddr, port: u16) -> bool {
    let addr = SocketAddr::new(ip, port);
    debug!("Try to connect to {addr}");

    // Tentative de connexion avec un timeout pour éviter de bloquer indéfiniment
    matches!(
        timeout(Duration::from_secs(2), TcpStream::connect(&addr)).await,
        Ok(Ok(_stream))
    )
}

/// Checks which IP addresses in a list are reachable on a given port.
///
/// # Arguments
///
/// * `ips` - A vector of IP addresses to check.
/// * `port` - The port to check for reachability.
///
/// # Returns
///
/// A vector containing only the IP addresses that are reachable on the specified port.
async fn is_reachables(ips: Vec<IpAddr>, port: u16) -> Vec<IpAddr> {
    let mut reachable_ips = Vec::new();

    for ip in ips {
        if is_reachable(ip, port).await {
            reachable_ips.push(ip);
        }
    }

    // Sort IP, localhost first, then private ipv4, then rest
    reachable_ips.sort_by(|a, b| {
        let a_loopback = a.is_loopback();
        let b_loopback = b.is_loopback();
        let a_is_private = if let IpAddr::V4(a) = a {
            a.is_private()
        } else {
            false
        };

        if a_loopback || (a_is_private && !b_loopback) {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    reachable_ips
}

/// Resolves a hostname to a list of IP addresses.
///
/// # Arguments
/// * `hostname` - The hostname to resolve.
///
/// # Returns
///
/// * `Vec<IpAddr>` - A list of resolved IP addresses.
#[must_use]
pub fn resolve_dns(hostname: &str) -> Vec<IpAddr> {
    match lookup_host(hostname) {
        Ok(iter) => iter.collect::<Vec<IpAddr>>(),
        Err(_) => Vec::new(),
    }
}

/// Asynchronously resolves a hostname to a list of IP addresses without blocking Tokio workers.
///
/// Internally uses the synchronous `dns_lookup::lookup_host` executed via `spawn_blocking`.
pub async fn resolve_dns_async(hostname: &str) -> Vec<IpAddr> {
    let host = hostname.to_string();
    // Optional timeout to avoid indefinitely stuck DNS resolution
    match timeout(
        Duration::from_secs(DNS_RESOLVE_TIMEOUT_SEC),
        spawn_blocking(move || resolve_dns(&host)),
    )
    .await
    {
        Ok(join_res) => match join_res {
            Ok(ips) => ips,
            Err(_) => Vec::new(),
        },
        Err(_) => Vec::new(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SocketAddrInformationSource {
    MDNS,
    DIRECT,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketAddrInformation {
    pub refresh_date: i64,
    pub hostname: String,
    pub port: u16,
    pub version: String,
    pub addresses: Vec<IpAddr>,
    pub is_online: bool,
    pub source: SocketAddrInformationSource,
}

impl FromRedisValue for SocketAddrInformation {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        let v: String = from_redis_value(v)?;

        debug!("Deserialize: {}", v);
        // Use serde_json to deserialize the string
        let v: SocketAddrInformation = match serde_json::from_str(&v) {
            Ok(v) => v,
            Err(e) => {
                return RedisResult::Err(RedisError::from((
                    redis::ErrorKind::TypeError,
                    "Invalid content",
                    e.to_string(),
                )));
            }
        };

        RedisResult::Ok(v)
    }
}

impl ToRedisArgs for SocketAddrInformation {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let Ok(v) = serde_json::to_string(self) else {
            return;
        };
        v.write_redis_args(out);
    }
}

/// The goal of this module is to provide a way to resolve a `SocketAddr` from a given hostname.
///
/// The resolver will use two methods to resolve the `SocketAddr`:
/// - mDNS (multicast DNS) to resolve the hostname to a `SocketAddr`.
/// - DNS (Domain Name System) to resolve the hostname to a `SocketAddr`.
///
/// The resolver will use the `mdns_sd` to listen for mDNS responses in continue, and will provide
/// a method to get the resolved `SocketAddr` if available.
///
/// If not available, the resolver will resolve DNS asynchronously via `spawn_blocking` around
/// the synchronous `dns_lookup::lookup_host` to avoid blocking the Tokio runtime.
#[derive(Clone)]
pub struct SocketAddrResolver {
    #[cfg(all(feature = "mdns", not(target_os = "freebsd")))]
    /// The mDNS service daemon used for multicast DNS resolution.
    mdns: ServiceDaemon,
    /// The Redis client used for DNS information storage and retrieval.
    redis: redis::Client,
}

impl SocketAddrResolver {
    /// Create a new `SocketAddrResolver` instance.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration containing Redis connection information.
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` if the resolver is successfully created.
    /// * `Err(eyre::Report)` if an error occurs during creation.
    ///
    /// # Errors
    ///
    /// Returns an error if the mDNS service daemon cannot be created (when the feature is enabled) or if the Redis client cannot be opened.
    ///
    /// # Panics
    ///
    /// This function will panic if the Redis client cannot be created from the provided URL (unwrap is used).
    pub fn new(client: redis::Client) -> Result<Self> {
        Ok(Self {
            #[cfg(all(feature = "mdns", not(target_os = "freebsd")))]
            mdns: ServiceDaemon::new()?,
            redis: client,
        })
    }

    /// Registers a service in the Redis database.
    ///
    /// # Arguments
    /// * `information` - The `SocketAddrInformation` to register.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the service is successfully registered.
    /// * `Err(eyre::Report)` if an error occurs during registration.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection to Redis fails or if the registration operation fails.
    pub async fn register_service(&self, information: &SocketAddrInformation) -> Result<()> {
        let mut con = self.redis.get_multiplexed_async_connection().await?;
        let hostname = information.hostname.clone();
        let _: () = con
            .hset(REDIS_WOODSTOCK_KEY_DNS, &hostname, information)
            .await?;

        if information.source == SocketAddrInformationSource::MDNS {
            return Ok(());
        }

        // Set timeout for the key direct
        let _: () = con
            .hexpire(
                REDIS_WOODSTOCK_KEY_DNS,
                DIRECT_DNS_UPDATE_INTERVAL,
                redis::ExpireOption::LT,
                &hostname,
            )
            .await?;

        Ok(())
    }

    /// Retrieves information about a hostname from the Redis database.
    ///
    /// # Arguments
    /// * `hostname` - The hostname to retrieve information for.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(SocketAddrInformation))` if the information is found.
    /// * `Ok(None)` if no information is found.
    /// * `Err(eyre::Report)` if an error occurs during retrieval.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection to Redis fails or if the retrieval operation fails.
    pub async fn get_informations(&self, hostname: &str) -> Result<Option<SocketAddrInformation>> {
        let mut con = self.redis.get_multiplexed_async_connection().await?;
        let result = con.hget(REDIS_WOODSTOCK_KEY_DNS, hostname).await?;

        Ok(result)
    }

    /// Updates the online status of a hostname in the Redis database.
    ///
    /// # Arguments
    /// * `hostname` - The hostname to update.
    /// * `is_online` - The new online status.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the status is successfully updated.
    /// * `Err(eyre::Report)` if an error occurs during the update.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection to Redis fails or if the update operation fails.
    pub async fn update_online_status(&self, hostname: &str, is_online: bool) -> Result<()> {
        let information = self.get_informations(hostname).await?;
        if let Some(mut information) = information {
            information.refresh_date = chrono::Utc::now().timestamp();
            information.is_online = is_online;
            self.register_service(&information).await?;
        }

        Ok(())
    }

    /// Resolves a hostname to a list of socket addresses.
    ///
    /// # Arguments
    /// * `hostname` - The hostname to resolve.
    /// * `default_port` - The default port to use if no port is specified.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<SocketAddr>)` if the resolution succeeds.
    /// * `Err(eyre::Report)` if an error occurs during resolution.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection to Redis fails or if the resolution operation fails.
    pub async fn resolve(&self, hostname: &str, default_port: u16) -> Result<Vec<SocketAddr>> {
        debug!("Resolve hostname: {}", hostname);
        let addresses = if let Some(socket_addr_info) = self.get_informations(hostname).await? {
            debug!(
                "Found hostname in cache: {hostname} (addresses: {:?})",
                socket_addr_info.addresses
            );
            let addresses =
                is_reachables(socket_addr_info.addresses.clone(), socket_addr_info.port).await;
            info!(
                "Found reachable addresses for host '{hostname}' in cache: {:?}",
                addresses
            );

            addresses
                .iter()
                .map(|ip| SocketAddr::new(*ip, socket_addr_info.port))
                .collect()
        } else {
            #[cfg(all(feature = "mdns", not(target_os = "freebsd")))]
            {
                info!("Resolve hostname with mdns: {}", hostname);
                let addresses = self.resolve_mdns(hostname, default_port).await;

                if let Some(addresses) = addresses {
                    info!("Found hostname with mdns: {}", hostname);
                    return Ok(addresses);
                }
            }

            info!("Resolve hostname with dns (spawn_blocking): {}", hostname);
            resolve_dns_async(hostname)
                .await
                .iter()
                .map(|ip| SocketAddr::new(*ip, default_port))
                .collect()
        };

        Ok(addresses)
    }

    /// Listens for incoming mDNS or DNS events and updates the resolver state.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the listener runs successfully.
    /// * `Err(eyre::Report)` if an error occurs while listening.
    ///
    /// # Errors
    ///
    /// Returns an error if the listener fails to start or encounters an error during execution.
    pub async fn listen(&self) -> Result<()> {
        #[cfg(all(feature = "mdns", not(target_os = "freebsd")))]
        {
            use tracing::{error, info};

            let receiver = self.mdns.browse(MDNS_SERVICE_NAME)?;

            // Use async receive to avoid blocking a Tokio worker thread.
            loop {
                match receiver.recv_async().await {
                    Ok(event) => match event {
                        ServiceEvent::SearchStarted(service_type) => {
                            info!("Search started: {service_type}");
                        }
                        ServiceEvent::SearchStopped(service_type) => {
                            info!("Search stopped: {service_type}");
                        }
                        ServiceEvent::ServiceFound(service_type, full_name) => {
                            info!("Service found: {service_type} {full_name}");
                        }
                        ServiceEvent::ServiceResolved(info) => {
                            info!("Service resolved: {:?}", info.get_fullname());
                            if let Err(err) = self.update_host(&info).await {
                                error!("Error while updating host: {:?}", err);
                            }
                        }
                        ServiceEvent::ServiceRemoved(service_type, full_name) => {
                            if service_type == MDNS_SERVICE_NAME {
                                info!("Service removed: {service_type} {full_name}");
                                let hostname = full_name.trim_end_matches(MDNS_SUFFIX);
                                if let Err(err) = self.update_online_status(hostname, false).await {
                                    error!("Error while updating host: {:?}", err);
                                }
                            }
                        }
                        _ => {}
                    },
                    Err(err) => {
                        // Channel closed or errored; log and exit the listener loop gracefully.
                        error!("mDNS browse channel error/closed: {err}");
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}

// MDNS part
#[cfg(all(feature = "mdns", not(target_os = "freebsd")))]
impl SocketAddrResolver {
    /// Updates the Redis database with information from an mDNS service.
    ///
    /// # Arguments
    /// * `info` - The resolved mDNS service information.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the information is successfully updated.
    /// * `Err(eyre::Report)` if an error occurs during the update.
    async fn update_host(&self, info: &ResolvedService) -> Result<()> {
        // Hostname without .local. suffix
        let hostname = info.get_fullname();
        let hostname = hostname.trim_end_matches(MDNS_SUFFIX);

        let port = info.get_port();

        let version = info
            .get_property("version")
            .map(mdns_sd::TxtProperty::val_str)
            .unwrap_or_default()
            .to_string();
        let addresses = info
            .get_addresses()
            .iter()
            .map(ScopedIp::to_ip_addr)
            .collect::<Vec<_>>();

        let socket_addr_info = SocketAddrInformation {
            refresh_date: chrono::Utc::now().timestamp(),
            hostname: hostname.to_string(),
            port,
            version,
            addresses,
            is_online: true,
            source: SocketAddrInformationSource::MDNS,
        };

        self.register_service(&socket_addr_info).await
    }

    /// Resolves a hostname using mDNS.
    ///
    /// # Arguments
    /// * `hostname` - The hostname to resolve.
    /// * `default_port` - The default port to use if no port is specified.
    ///
    /// # Returns
    ///
    /// * `Some(Vec<SocketAddr>)` if the resolution succeeds.
    /// * `None` if the resolution fails.
    async fn resolve_mdns(&self, hostname: &str, default_port: u16) -> Option<Vec<SocketAddr>> {
        let mdns_recv = self
            .mdns
            .resolve_hostname(&format!("{hostname}{MDNS_SUFFIX}"), Some(MDNS_TIMEOUT_MSEC));

        if let Ok(recv) = mdns_recv {
            let info = recv.recv_async().await;
            if let Ok(HostnameResolutionEvent::AddressesFound(_, info)) = info {
                let info = info
                    .into_iter()
                    .map(|ip| ip.to_ip_addr())
                    .collect::<Vec<_>>();
                let addresses = is_reachables(info, default_port).await;

                return Some(
                    addresses
                        .iter()
                        .map(|ip| SocketAddr::new(*ip, default_port))
                        .collect(),
                );
            }
        }

        None
    }
}

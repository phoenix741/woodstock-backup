#[cfg(feature = "mdns")]
use crate::config::{MDNS_SERVICE_NAME, MDNS_SUFFIX, MDNS_TIMEOUT_MSEC};
#[cfg(feature = "mdns")]
use mdns_sd::{HostnameResolutionEvent, ServiceDaemon, ServiceEvent, ServiceInfo};

use dns_lookup::lookup_host;
use eyre::Result;
use log::{debug, info};
use redis::{
    from_redis_value, AsyncCommands, FromRedisValue, RedisError, RedisResult, ToRedisArgs, Value,
};
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use tokio::{net::TcpStream, time::timeout};

use crate::config::{Context, REDIS_WOODSTOCK_KEY_DNS};

const DIRECT_DNS_UPDATE_INTERVAL: i64 = 120;

async fn is_reachable(ip: IpAddr, port: u16) -> bool {
    let addr = SocketAddr::new(ip, port);

    // Tentative de connexion avec un timeout pour éviter de bloquer indéfiniment
    matches!(
        timeout(Duration::from_secs(2), TcpStream::connect(&addr)).await,
        Ok(Ok(_stream))
    )
}

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

pub fn resolve_dns(hostname: &str) -> Vec<IpAddr> {
    lookup_host(hostname).ok().unwrap_or_default()
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
        let v = if let Ok(v) = serde_json::to_string(self) {
            v
        } else {
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
/// The resolver will use the mdns_sd to listen for mDNS responses in continue, and will provide
/// a method to get the resolved `SocketAddr` if available.
///
/// If not available, the resolver will use the tokio::net::lookup_host to resolve the hostname
#[derive(Clone)]
pub struct SocketAddrResolver {
    #[cfg(feature = "mdns")]
    mdns: ServiceDaemon,
    redis: redis::Client,
}

impl SocketAddrResolver {
    /// Create a new `SocketAddrResolver` instance.
    pub fn new(context: &Context) -> Result<Self> {
        let redis_url = format!(
            "redis://{}:{}",
            context.config.redis.host, context.config.redis.port
        );
        info!("Connect to Redis URL for DNS resolution: {}", redis_url);

        let client = redis::Client::open(redis_url).unwrap();
        Ok(Self {
            #[cfg(feature = "mdns")]
            mdns: ServiceDaemon::new()?,
            redis: client,
        })
    }

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

    pub async fn get_informations(&self, hostname: &str) -> Result<Option<SocketAddrInformation>> {
        let mut con = self.redis.get_multiplexed_async_connection().await?;
        let result = con.hget(REDIS_WOODSTOCK_KEY_DNS, hostname).await?;

        Ok(result)
    }

    pub async fn update_online_status(&self, hostname: &str, is_online: bool) -> Result<()> {
        let information = self.get_informations(hostname).await?;
        if let Some(mut information) = information {
            information.refresh_date = chrono::Utc::now().timestamp();
            information.is_online = is_online;
            self.register_service(&information).await?;
        }

        Ok(())
    }

    pub async fn resolve(&self, hostname: &str, default_port: u16) -> Result<Vec<SocketAddr>> {
        debug!("Resolve hostname: {}", hostname);
        let addresses = if let Some(socket_addr_info) = self.get_informations(hostname).await? {
            debug!("Found hostname in cache: {}", hostname);
            let addresses =
                is_reachables(socket_addr_info.addresses.clone(), socket_addr_info.port).await;

            addresses
                .iter()
                .map(|ip| SocketAddr::new(*ip, socket_addr_info.port))
                .collect()
        } else {
            #[cfg(feature = "mdns")]
            {
                debug!("Resolve hostname with mdns: {}", hostname);
                let addresses = self.resolve_mdns(hostname, default_port).await;

                if let Some(addresses) = addresses {
                    debug!("Found hostname with mdns: {}", hostname);
                    return Ok(addresses);
                }
            }

            debug!("Resolve hostname with dns: {}", hostname);
            resolve_dns(hostname)
                .iter()
                .map(|ip| SocketAddr::new(*ip, default_port))
                .collect()
        };

        Ok(addresses)
    }

    pub async fn listen(&self) -> Result<()> {
        #[cfg(feature = "mdns")]
        {
            use log::{error, info};

            let receiver = self.mdns.browse(MDNS_SERVICE_NAME)?;

            while let Ok(event) = receiver.recv() {
                match event {
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
                        info!("Service removed: {service_type} {full_name}");
                        if service_type == MDNS_SERVICE_NAME {
                            let hostname = full_name.trim_end_matches(MDNS_SUFFIX);
                            if let Err(err) = self.update_online_status(hostname, false).await {
                                error!("Error while updating host: {:?}", err);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

// MDNS part
#[cfg(feature = "mdns")]
impl SocketAddrResolver {
    async fn update_host(&self, info: &ServiceInfo) -> Result<()> {
        // Hostname without .local. suffix
        let hostname = info.get_fullname();
        let hostname = hostname.trim_end_matches(MDNS_SUFFIX);

        let port = info.get_port();

        let version = info
            .get_property("version")
            .map(|version| version.val_str())
            .unwrap_or_default()
            .to_string();
        let addresses = info.get_addresses().iter().cloned().collect::<Vec<_>>();

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

    async fn resolve_mdns(&self, hostname: &str, default_port: u16) -> Option<Vec<SocketAddr>> {
        let mdns_recv = self.mdns.resolve_hostname(
            &format!("{}{}", hostname, MDNS_SUFFIX),
            Some(MDNS_TIMEOUT_MSEC),
        );

        if let Ok(recv) = mdns_recv {
            let info = recv.recv_async().await;
            if let Ok(HostnameResolutionEvent::AddressesFound(_, info)) = info {
                let info = info.into_iter().collect::<Vec<_>>();
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

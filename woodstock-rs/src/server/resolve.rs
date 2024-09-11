use dns_lookup::lookup_host;
use eyre::Result;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::Duration,
};
use tokio::{net::TcpStream, sync::Mutex, time::timeout};

use crate::config::MDNS_SERVICE_NAME;

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

#[derive(Debug, Clone)]
pub struct SocketAddrInformation {
    pub hostname: String,
    pub port: u16,
    pub version: String,
    pub addresses: Vec<IpAddr>,
    pub is_online: bool,
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
    host_map: Arc<Mutex<HashMap<String, SocketAddrInformation>>>,
}

impl SocketAddrResolver {
    /// Create a new `SocketAddrResolver` instance.
    pub fn new() -> Result<Self> {
        Ok(Self {
            host_map: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    async fn update_host(&self, info: &ServiceInfo) {
        // Hostname without .local. suffix
        let hostname = info.get_hostname();
        let hostname = hostname.trim_end_matches(".local.");

        let port = info.get_port();

        let version = info
            .get_property("version")
            .map(|version| version.val_str())
            .unwrap_or_default()
            .to_string();
        let addresses = info.get_addresses().iter().cloned().collect::<Vec<_>>();

        let socket_addr_info = SocketAddrInformation {
            hostname: hostname.to_string(),
            port,
            version,
            addresses,
            is_online: true,
        };

        let mut host_map = self.host_map.lock().await;
        host_map.insert(hostname.to_string(), socket_addr_info);
    }

    async fn update_online_status(&self, hostname: &str, is_online: bool) {
        let mut host_map = self.host_map.lock().await;
        if let Some(socket_addr_info) = host_map.get_mut(hostname) {
            socket_addr_info.is_online = is_online;
        }
    }

    pub async fn listen(&self) -> Result<()> {
        let mdns = ServiceDaemon::new()?;
        let receiver = mdns.browse(MDNS_SERVICE_NAME)?;

        while let Ok(event) = receiver.recv() {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    self.update_host(&info).await;
                }
                ServiceEvent::ServiceRemoved(service_type, full_name) => {
                    let service_type = format!(".{}", service_type);
                    let hostname = full_name.trim_end_matches(&service_type);
                    self.update_online_status(hostname, false).await;
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub async fn resolve(&self, hostname: &str, default_port: u16) -> Option<Vec<SocketAddr>> {
        let host_map = self.host_map.lock().await;
        let addresses = if let Some(socket_addr_info) = host_map.get(hostname) {
            let addresses =
                is_reachables(socket_addr_info.addresses.clone(), socket_addr_info.port).await;

            addresses
                .iter()
                .map(|ip| SocketAddr::new(*ip, socket_addr_info.port))
                .collect()
        } else {
            resolve_dns(hostname)
                .iter()
                .map(|ip| SocketAddr::new(*ip, default_port))
                .collect()
        };

        Some(addresses)
    }

    pub async fn get_informations(&self, hostname: &str) -> Option<SocketAddrInformation> {
        let host_map = self.host_map.lock().await;
        host_map.get(hostname).cloned()
    }
}

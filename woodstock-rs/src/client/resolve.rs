use eyre::Result;
use log::info;
use mdns_sd::{ServiceDaemon, ServiceInfo};

use crate::config::MDNS_SERVICE_NAME;

use super::config::ClientConfig;

pub fn mdns_responder(config: &ClientConfig) -> Result<ServiceDaemon> {
    let addr: std::net::SocketAddr = config.bind.parse()?;
    let port = addr.port();

    let properties = [("version", ClientConfig::version())];

    let mdns = ServiceDaemon::new().expect("Failed to create daemon");
    let my_service = ServiceInfo::new(
        MDNS_SERVICE_NAME,
        &config.hostname,
        &format!("{}.local.", &config.hostname),
        "",
        port,
        &properties[..],
    )
    .expect("Failed to create service info for mDNS")
    .enable_addr_auto();

    // Register with the daemon, which publishes the service.
    mdns.register(my_service)
        .expect("Failed to register our service");
    info!("Service mDNS enregistré et disponible.");

    Ok(mdns)
}

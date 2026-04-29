//! Windows Firewall rule management for the Woodstock client daemon.

use crate::winfw::{
    create_firewall_rule, delete_firewall_rule, rule_exists, Actions, FwRule, Protocols,
};
use eyre::Result;
use std::net::SocketAddr;
use woodstock::config::DEFAULT_PORT;

fn get_port_from_address(address: &str) -> u16 {
    match address.parse::<SocketAddr>() {
        Ok(socket_addr) => socket_addr.port(),
        Err(_) => DEFAULT_PORT,
    }
}

pub fn add_firewall_rule(bind: &str) -> Result<()> {
    let port = get_port_from_address(bind);

    // Règle pour autoriser le trafic TCP entrant sur le port spécifique
    let tcp_rule_name = "Woodstock Client Daemon TCP";
    if rule_exists(tcp_rule_name)? {
        delete_firewall_rule(tcp_rule_name)?;
    }

    let tcp_rule = FwRule {
        name: tcp_rule_name.to_string(),
        description: format!("Allow incoming TCP traffic on port {}", port),
        local_ports: port.to_string(),
        protocol: Protocols::Tcp,
        action: Actions::Allow,
        enabled: true,
        ..FwRule::default()
    };
    create_firewall_rule(&tcp_rule)?;

    // Règle pour autoriser le trafic UDP entrant et sortant sur le port mDNS (5353)
    let udp_rule_name = "Woodstock Client Daemon mDNS";
    if rule_exists(udp_rule_name)? {
        delete_firewall_rule(udp_rule_name)?;
    }

    let udp_rule = FwRule {
        name: udp_rule_name.to_string(),
        description: "Allow incoming and outgoing UDP traffic on port 5353 for mDNS".to_string(),
        local_ports: "5353".to_string(),
        protocol: Protocols::Udp,
        action: Actions::Allow,
        enabled: true,
        ..FwRule::default()
    };
    create_firewall_rule(&udp_rule)?;

    Ok(())
}

pub fn remove_firewall_rule() -> Result<()> {
    // Supprimer la règle TCP
    let tcp_rule_name = "Woodstock Client Daemon TCP";
    delete_firewall_rule(tcp_rule_name)?;

    // Supprimer la règle UDP
    let udp_rule_name = "Woodstock Client Daemon mDNS";
    delete_firewall_rule(udp_rule_name)?;

    Ok(())
}

//! This module provides network and mDNS resolution commands for Woodstock backups.
//!
//! It includes functions to discover and print network addresses for backup servers using multicast DNS, facilitating dynamic service discovery in local networks.
//!
//! # Errors
//!
//! Functions in this module may return errors if mDNS resolution fails, if the network is unreachable, or if terminal output fails.
//!
//! # Panics
//!
//! Some functions may panic if system resources are unavailable or if I/O operations fail unexpectedly.

use console::Term;
use eyre::Result;
use tokio::select;
use tokio::time::sleep;
use tracing::info;

use crate::commands::CliServiceState;

/// Resolves the given hostname using mDNS and prints its network addresses.
///
/// # Arguments
///
/// * `config` - The Woodstock configuration containing network settings.
/// * `hostname` - The hostname to resolve via mDNS.
///
/// # Errors
///
/// Returns an error if mDNS resolution fails or if writing to the terminal fails.
///
/// # Panics
///
/// This function does not explicitly panic.
pub async fn resolve_mdns(state: CliServiceState, hostname: &str) -> Result<()> {
    let term = Term::stdout();

    let token = crate::cancel::cancellation_token_with_ctrl_c();
    let cloned_token = token.clone();

    let Some(resolver) = state.resolver else {
        return Err(eyre::eyre!("mDNS resolver is not configured"));
    };

    let listener = resolver.clone();
    let handle = tokio::spawn(async move {
        select! {
          () = cloned_token.cancelled() => {
            5
          }
          _ = listener.listen() => {
            99
          }
        }
    });

    loop {
        info!("Search for hostname: {hostname}");
        if let Some(information) = resolver.get_informations(hostname).await? {
            term.write_line(&format!("Hostname: {}", information.hostname))?;
            term.write_line(&format!(
                "Addresses: {}",
                information
                    .addresses
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", "),
            ))?;
            term.write_line(&format!("Port: {}", information.port))?;
            term.write_line(&format!("Version: {}", information.version))?;
            break;
        }

        select! {
            () = token.cancelled() => {
                info!("mDNS resolution cancelled by user");
                break;
            }
            () = sleep(std::time::Duration::from_secs(10)) => {}
        }
    }

    tokio::spawn(async move {
        token.cancel();
    });

    handle.await?;

    Ok(())
}

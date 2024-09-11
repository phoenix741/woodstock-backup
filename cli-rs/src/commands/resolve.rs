use std::sync::Arc;

use console::Term;
use eyre::Result;
use log::info;
use tokio::select;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use woodstock::{config::Context, server::resolve::SocketAddrResolver};

pub async fn resolve_mdns(_ctxt: &Context, hostname: &str) -> Result<()> {
    let term = Term::stdout();

    let resolver = Arc::new(SocketAddrResolver::new()?);

    let token = CancellationToken::new();
    let cloned_token = token.clone();

    let listener = resolver.clone();
    let handle = tokio::spawn(async move {
        select! {
          _ = cloned_token.cancelled() => {
            5
          }
          _ = listener.listen() => {
            99
          }
        }
    });

    loop {
        info!("Search for hostname: {hostname}");
        if let Some(information) = resolver.get_informations(hostname).await {
            term.write_line(&format!("Hostname: {}", information.hostname))?;
            term.write_line(&format!(
                "Addresses: {}",
                information
                    .addresses
                    .iter()
                    .map(|addr| addr.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ))?;
            term.write_line(&format!("Port: {}", information.port))?;
            term.write_line(&format!("Version: {}", information.version))?;
            break;
        }

        sleep(std::time::Duration::from_secs(10)).await;
    }

    tokio::spawn(async move {
        token.cancel();
    });

    handle.await?;

    Ok(())
}

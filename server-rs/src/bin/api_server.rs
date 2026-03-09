//! Public API Server binary
//!
//! This binary implements the public-facing REST and GraphQL API that serves
//! the Vue.js frontend, replacing the NestJS api application.

use eyre::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, error, info, Instrument};
use woodstock::config::Configuration;

use woodstock_server_rs::api::{routes::create_router, ApiServerConfig, ApiServerState};
use woodstock_server_rs::logger::init_logging;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize crypto provider for rustls before any TLS operations
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| eyre::eyre!("Failed to install crypto provider"))?;

    let woodstock_config = Arc::new(Configuration::default());

    init_logging(&woodstock_config.path.logs_path, "server_api.log")?;
    info!(
        "Starting Woodstock Public API Server with BACKUP_PATH: {}",
        woodstock_config.path.backup_path.display()
    );

    // Load configuration
    let config = ApiServerConfig::default();

    // Create application state
    let state = match ApiServerState::new(woodstock_config.clone()).await {
        Ok(state) => state,
        Err(e) => {
            error!("Failed to initialize application state: {}", e);
            return Err(e);
        }
    };

    debug!("Generate certificate");
    state.certificate_service.generate_certificate().await?;

    // Start mDNS/DNS listener in background
    let _listener_resolver = state.resolver.clone();
    tokio::spawn(
        async move {
            if let Err(e) = _listener_resolver.listen().await {
                error!("Resolver listen error: {e}");
            }
        }
        .in_current_span(),
    );

    // Create router
    let app = create_router(state, config.static_path.clone());

    // Create server address
    let bind_addr = config.api_address();
    info!("API server listening on {}", bind_addr);
    info!(
        "Swagger UI available at http://localhost:{}/api-docs",
        config.port
    );
    let bind_addr: SocketAddr = bind_addr.parse()?;

    // Create listener and serve
    axum_server::bind(bind_addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

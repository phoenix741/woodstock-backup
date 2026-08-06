//! Client API Server Binary
//!
//! mTLS authenticated API server for backup clients.
//! This server provides endpoints that exactly match the NestJS clientApi structure:
//! - POST /api/hosts/:name/client - Register client network information

use axum::{middleware, routing::post, Router};
use axum_server::tls_rustls::RustlsConfig;
use color_eyre::eyre::{eyre, Result, WrapErr};
use rustls::{
    pki_types::{CertificateDer, PrivateKeyDer},
    server::WebPkiClientVerifier,
    RootCertStore,
};
use rustls_pemfile;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::fs;
use tower_http::trace::TraceLayer;
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use woodstock::config::Configuration;
use woodstock_server_rs::{
    client_api::{
        auth::client_cert_middleware,
        handlers::{register_client, ClientApiState},
        middleware::client_cert_logging_middleware,
        ClientApiConfig, ClientApiDoc, ClientCertAcceptor,
    },
    logger::init_logging,
};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let client_config = Arc::new(ClientApiConfig::from_env());
    let woodstock_config = Arc::new(Configuration::default());

    let state = ClientApiState::new(woodstock_config.clone()).await?;

    state
        .certificate_service
        .generate_https_certificate()
        .await?;

    // Initialize crypto provider for rustls before any TLS operations
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| eyre::eyre!("Failed to install crypto provider"))?;

    init_logging(&woodstock_config.path.logs_path, "client_api.log")?;
    info!("Starting Client API Server");

    // Build TLS configuration
    let tls_config = build_tls_config(&woodstock_config).await?;

    // Build application router
    let app = build_app_router(state);

    let bind_addr = client_config.client_api_address();
    info!("Client API server listening on https://{}", bind_addr);
    let bind_addr: SocketAddr = bind_addr.parse()?;

    // Start HTTPS server with mTLS using our custom acceptor
    axum_server::bind(bind_addr)
        .acceptor(ClientCertAcceptor::new(
            axum_server::tls_rustls::RustlsAcceptor::new(tls_config),
        ))
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/// Build the application router with all routes and middleware
fn build_app_router(state: ClientApiState) -> Router {
    let mut swagger_doc = ClientApiDoc::openapi();
    swagger_doc.info.version = Configuration::version();
    Router::new()
        // Swagger UI endpoint for API documentation (development only)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", swagger_doc))
        .merge(
            Router::new()
                // Client registration endpoint (matches NestJS structure exactly)
                .route("/api/hosts/{name}/client", post(register_client))
                // Add state
                .with_state(state)
                // 2. Request logging
                .layer(middleware::from_fn(client_cert_logging_middleware))
                // 3. Client certificate validation (applied first)
                .layer(middleware::from_fn(client_cert_middleware))
                // 4. Tracing middleware for HTTP requests
                .layer(TraceLayer::new_for_http()),
        )
}

/// Build TLS configuration with client certificate verification
async fn build_tls_config(config: &Configuration) -> Result<RustlsConfig> {
    let cert_dir = &config.path.certificates_path;
    let ca_cert = cert_dir.join("rootCA.pem");
    let server_cert = cert_dir.join("https.pem");
    let server_key = cert_dir.join("https.key");

    // Load server certificate
    let cert_pem = fs::read_to_string(&server_cert)
        .await
        .wrap_err_with(|| format!("Failed to read server cert {}", server_cert.display()))?;

    let certs: Vec<CertificateDer> = rustls_pemfile::certs(&mut cert_pem.as_bytes())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| eyre!("Failed to parse server cert: {}", e))?;

    // Load server private key
    let private_key = fs::read_to_string(&server_key)
        .await
        .wrap_err_with(|| format!("Failed to read server key {}", server_key.display()))?;

    let private_key: PrivateKeyDer = rustls_pemfile::private_key(&mut private_key.as_bytes())
        .map_err(|e| eyre!("Failed to parse server key: {}", e))?
        .ok_or_else(|| eyre!("No private key found"))?;

    // Load CA certificate for client verification
    let ca_certs = fs::read_to_string(&ca_cert)
        .await
        .wrap_err("Failed to read CA cert")?;

    let ca_certs: Vec<CertificateDer> = rustls_pemfile::certs(&mut ca_certs.as_bytes())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| eyre!("Failed to parse CA cert: {}", e))?;

    // Build root cert store for client verification
    let mut root_store = RootCertStore::empty();
    for cert in ca_certs {
        root_store
            .add(cert)
            .map_err(|e| eyre!("Failed to add CA cert: {}", e))?;
    }

    // Create client cert verifier
    let client_verifier = WebPkiClientVerifier::builder(root_store.into())
        .allow_unauthenticated()
        .build()
        .map_err(|e| eyre!("Failed to build client verifier: {}", e))?;

    // Build TLS server config with client cert verification
    let tls_config = rustls::ServerConfig::builder()
        .with_client_cert_verifier(client_verifier)
        .with_single_cert(certs, private_key)
        .map_err(|e| eyre!("Failed to build TLS config: {}", e))?;

    Ok(RustlsConfig::from_config(Arc::new(tls_config)))
}

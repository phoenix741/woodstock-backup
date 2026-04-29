//! Client API Authentication Module
//!
//! Handles mTLS certificate validation and authentication for the client API server.
//! This module provides:
//! - Client certificate validation against the CA
//! - CN extraction from client certificates
//! - Integration with woodstock-rs HostsService for authorization
//! - Authentication guard for protecting endpoints
//!
//! Based on: https://github.com/JorianWoltjer/axum-mtls-client-authentication

use crate::api::error::ApiError;
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::{AddExtension, Next},
    response::Response,
    Extension,
};
use axum_server::{accept::Accept, tls_rustls::RustlsAcceptor};
use futures_util::future::BoxFuture;
use rustls::pki_types::CertificateDer;
use std::io;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::server::TlsStream;
use tower::Layer;
use tracing::{debug, error, warn};
use woodstock::{self, config::Hosts};
use x509_parser::{certificate::X509Certificate, prelude::FromDer};

/// Extension key for storing the client CN in the request
pub const CLIENT_CN_EXTENSION_KEY: &str = "client_cn";

/// Authentication guard extractor for client certificate information
///
/// Use this in your handlers to ensure the request has a valid client certificate.
///
/// # Example
/// ```rust,no_run
/// use axum::{Json, response::Response, http::StatusCode};
/// use woodstock_server_rs::client_api::ClientAuth;
///
/// async fn my_handler(cert_info: ClientAuth) -> Result<Json<&'static str>, StatusCode> {
///     println!("Client CN: {}", cert_info.common_name);
///     Ok(Json("success"))
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ClientAuth {
    pub common_name: String,
    pub subject_dn: String,
    pub issuer_dn: String,
}

impl std::fmt::Display for ClientAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ClientAuth {{ CN: {}, Subject: {}, Issuer: {} }}",
            self.common_name, self.subject_dn, self.issuer_dn
        )
    }
}

/// TLS data captured from the connection including peer certificates
#[derive(Debug, Clone)]
pub struct TlsData {
    /// Client certificates provided during TLS handshake
    pub peer_certificates: Option<Vec<CertificateDer<'static>>>,
}

/// Custom TLS acceptor that captures client certificate information
#[derive(Debug, Clone)]
pub struct ClientCertAcceptor {
    inner: RustlsAcceptor,
}

impl ClientCertAcceptor {
    /// Create a new client certificate acceptor wrapping a RustlsAcceptor
    pub fn new(inner: RustlsAcceptor) -> Self {
        Self { inner }
    }
}

impl<I, S> Accept<I, S> for ClientCertAcceptor
where
    I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    S: Send + 'static,
{
    type Stream = TlsStream<I>;
    type Service = AddExtension<S, TlsData>;
    type Future = BoxFuture<'static, io::Result<(Self::Stream, Self::Service)>>;

    fn accept(&self, stream: I, service: S) -> Self::Future {
        let acceptor = self.inner.clone();

        Box::pin(async move {
            let (tls_stream, service) = acceptor.accept(stream, service).await?;
            let server_conn = tls_stream.get_ref().1;

            // Extract peer certificates from the TLS connection
            let tls_data = TlsData {
                peer_certificates: server_conn.peer_certificates().map(From::from),
            };

            debug!(
                "TLS connection established with {} peer certificates",
                tls_data
                    .peer_certificates
                    .as_ref()
                    .map_or(0, |certs| certs.len())
            );

            // Use Extension layer to inject TLS data into every request
            let service = Extension(tls_data).layer(service);

            Ok((tls_stream, service))
        })
    }
}

/// Extract and validate client certificate from request
///
/// This function is called by middleware to extract client certificate information.
pub fn extract_and_validate_client_cert(tls_data: &TlsData) -> Result<ClientAuth, ApiError> {
    // Extract client certificate from TLS data
    if let Some(peer_certificates) = &tls_data.peer_certificates {
        if let Some(cert_der) = peer_certificates.first() {
            return parse_client_certificate(cert_der);
        }
    }

    // No client certificate available
    Err(ApiError::Unauthorized(
        "No client certificate provided".to_string(),
    ))
}

/// Middleware to extract and validate client certificate information
///
/// This middleware:
/// 1. Extracts client certificate from TLS connection (or test headers)
/// 2. Validates the certificate chain
/// 3. Stores ClientCertInfo in request extensions for use by handlers
pub async fn client_cert_middleware(
    Extension(tls_data): Extension<TlsData>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    debug!("Processing client certificate authentication middleware");

    // Extract client certificate information
    let cert_info = match extract_and_validate_client_cert(&tls_data) {
        Ok(info) => info,
        Err(err) => {
            error!("Client certificate extraction failed: {}", err);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    debug!("Successfully extracted client certificate: {}", cert_info);

    // Store the certificate info in request extensions for handlers to use
    let mut req = req;
    req.extensions_mut().insert(cert_info);

    // Continue to the next handler
    Ok(next.run(req).await)
}

/// Parse client certificate from DER format
fn parse_client_certificate(cert_der: &CertificateDer) -> Result<ClientAuth, ApiError> {
    // Parse the certificate using x509_parser
    let (_, cert) = X509Certificate::from_der(cert_der.as_ref())
        .map_err(|e| ApiError::Unauthorized(format!("Failed to parse certificate: {}", e)))?;

    // Extract the subject and issuer DNs
    let subject_dn = cert.subject().to_string();
    let issuer_dn = cert.issuer().to_string();

    // Extract the common name from the subject
    let common_name = cert
        .subject()
        .iter_common_name()
        .next()
        .ok_or_else(|| {
            ApiError::Unauthorized("Missing common name in client certificate".to_string())
        })?
        .as_str()
        .map_err(|_| {
            ApiError::Unauthorized("Invalid common name in client certificate".to_string())
        })?
        .to_string();

    debug!(
        "Parsed client certificate - CN: {}, Subject: {}, Issuer: {}",
        common_name, subject_dn, issuer_dn
    );

    Ok(ClientAuth {
        common_name,
        subject_dn,
        issuer_dn,
    })
}

/// Validate client certificate against woodstock-rs HostsService
///
/// This function integrates with the woodstock-rs Hosts service to verify
/// that the client with the given CN is authorized to connect.
pub async fn validate_host_exists(cn: &str, hosts_service: &Hosts) -> Result<bool, ApiError> {
    debug!("Validating client authorization for CN: {}", cn);

    // Check if the host exists in the system
    match hosts_service.get_host(cn).await {
        Ok(_host_config) => {
            debug!("Host {} found in configuration", cn);

            // Since the host exists in the configuration, it's authorized
            debug!("Host {} is authorized to connect", cn);
            Ok(true)
        }
        Err(err) => {
            warn!("Host {} authorization failed: {}", cn, err);
            // Host not found or configuration error
            Ok(false)
        }
    }
}

impl<S> axum::extract::FromRequestParts<S> for ClientAuth
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        // Wrap sync extraction in async-block
        let result = parts
            .extensions
            .get::<ClientAuth>()
            .cloned()
            .ok_or(StatusCode::UNAUTHORIZED);
        async move { result }
    }
}

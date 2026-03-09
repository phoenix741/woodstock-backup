//! Client API middleware for certificate extraction and validation
//!
//! This module provides middleware functions for handling mTLS client certificate
//! authentication in the client API server.

use axum::{extract::Request, middleware::Next, response::Response};
use tracing::{debug, info, warn};

use crate::client_api::ClientAuth;

/// Middleware to extract and validate client certificates
///
/// This middleware is responsible for:
/// 1. Extracting the client certificate from the TLS connection
/// 2. Validating the certificate against the configured CA
/// This middleware is moved to auth.rs and uses the GitHub pattern
/// with Extension<TlsData> for proper certificate extraction.
///
/// Use client_cert_middleware from auth module instead.

/// Middleware for logging client certificate information
///
/// This middleware logs information about the client certificate for debugging
/// and audit purposes. It should be placed after the certificate extraction
/// middleware.
pub async fn client_cert_logging_middleware(req: Request, next: Next) -> Response {
    let start = std::time::Instant::now();

    // Check if we have client certificate information
    if let Some(cert_info) = req.extensions().get::<ClientAuth>() {
        info!(
            "Client request with certificate: CN={}, Subject={}",
            cert_info.common_name, cert_info.subject_dn
        );
    } else {
        warn!("Client request without certificate information");
    }

    let response = next.run(req).await;
    let duration = start.elapsed();

    debug!("Client API request completed in {:?}", duration);

    response
}

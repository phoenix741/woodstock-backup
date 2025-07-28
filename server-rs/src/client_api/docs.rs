//! OpenAPI documentation for the Client API
//!
//! This module defines the OpenAPI/Swagger specification for all client API endpoints.

use utoipa::OpenApi;

use crate::client_api::dto::{InterfaceInformation, Ipv4Addr, Ipv6Addr, RegisterClient};

/// OpenAPI specification for the Client API
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::client_api::handlers::register_client,
    ),
    components(
        schemas(RegisterClient, InterfaceInformation, Ipv4Addr, Ipv6Addr)
    ),
    tags(
        (name = "client-api", description = "mTLS authenticated endpoints for backup clients")
    ),
    info(
        title = "Woodstock Client API",
        description = "Secure mTLS-authenticated API for backup client registration and management",
        license(
            name = "MIT License",
            url = "https://opensource.org/license/mit"
        )
    ),
    servers(
        (url = "https://localhost:8443", description = "Development server")
    )
)]
pub struct ClientApiDoc;

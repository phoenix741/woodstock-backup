//! Client API handlers for processing client registration requests

use crate::api::error::ApiError;
use crate::{
    api::services::CertificateService,
    client_api::{
        auth::{validate_host_exists, ClientAuth},
        dto::RegisterClient,
        validation::ValidatedJson,
    },
};
use axum::{
    extract::{Path, State},
    response::Json,
};
use eyre::Result as EyreResult;
use serde_json::Value;
use std::{net::IpAddr, sync::Arc};
use tracing::{debug, info, warn};
use woodstock::{
    config::{Configuration, Hosts, Scheduler},
    server::resolve::{SocketAddrInformation, SocketAddrInformationSource, SocketAddrResolver},
};

/// Application state for client API
#[derive(Clone)]
pub struct ClientApiState {
    pub woodstock_config: Arc<woodstock::config::Configuration>,
    pub scheduler: Arc<Scheduler>,
    pub hosts: Arc<Hosts>,
    pub resolver: Arc<SocketAddrResolver>,
    pub certificate_service: Arc<CertificateService>,
}

impl ClientApiState {
    /// Create new API server state
    pub async fn new(config: Arc<Configuration>) -> EyreResult<Self> {
        let redis_url = config.redis_url();
        info!("Connect to Redis URL for DNS resolution: {}", redis_url);
        let redis_client = redis::Client::open(redis_url)?;

        let scheduler = Arc::new(Scheduler::new(config.clone()));
        let hosts = Arc::new(Hosts::new(config.clone(), scheduler.clone()));
        let resolver = Arc::new(SocketAddrResolver::new(redis_client.clone())?);
        let certificate_service = Arc::new(CertificateService::new(config.clone()));

        Ok(Self {
            woodstock_config: config.clone(),

            hosts,
            scheduler,
            resolver,
            certificate_service,
        })
    }
}

/// Handler for POST /api/hosts/:name/client endpoint
///
/// This endpoint matches the NestJS implementation and allows backup clients
/// to register their network information for DNS resolution.
#[utoipa::path(
    post,
    path = "/api/hosts/{hostname}/client",
    tag = "client-api",
    operation_id = "register_client",
    request_body = RegisterClient,
    params(
        ("hostname" = String, Path, description = "Hostname of the backup client (must match certificate CN)")
    ),
    responses(
        (status = 200, description = "Client successfully registered", body = serde_json::Value),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Client certificate authentication failed"),
        (status = 403, description = "Client not authorized for this hostname"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn register_client(
    Path(hostname): Path<String>,
    client_auth: ClientAuth, // Client certificate information from mTLS middleware
    State(state): State<ClientApiState>,
    ValidatedJson(body): ValidatedJson<RegisterClient>, // Validated JSON body
) -> Result<Json<Value>, ApiError> {
    info!("Client registration request from {}: {:?}", hostname, body);

    // Verify that the client certificate CN matches the hostname parameter
    if client_auth.common_name != hostname {
        warn!(
            "Client certificate CN '{}' does not match hostname parameter '{}'",
            client_auth.common_name, hostname
        );
        return Err(ApiError::Forbidden(format!(
            "You are not allowed to register this host: {}",
            hostname
        )));
    }

    // Additional validation: check if the host is authorized in woodstock configuration
    if !validate_host_exists(&client_auth.common_name, &state.hosts).await? {
        warn!(
            "Host '{}' is not authorized in woodstock configuration",
            client_auth.common_name
        );
        return Err(ApiError::Forbidden(format!(
            "Host '{}' is not authorized",
            client_auth.common_name
        )));
    }

    // Convert interface information to IP addresses
    let addresses: Vec<IpAddr> = body
        .addresses
        .iter()
        .filter_map(|interface| {
            let mut ips = Vec::new();

            // Add IPv4 address if present
            if let Some(ipv4) = &interface.ipv4 {
                if let Ok(ip) = ipv4.addr.parse::<IpAddr>() {
                    ips.push(ip);
                }
            }

            // Add IPv6 address if present
            if let Some(ipv6) = &interface.ipv6 {
                if let Ok(ip) = ipv6.addr.parse::<IpAddr>() {
                    ips.push(ip);
                }
            }

            if ips.is_empty() {
                None
            } else {
                Some(ips)
            }
        })
        .flatten()
        .collect();

    debug!(
        "Extracted {} IP addresses from client registration: {:?}",
        addresses.len(),
        addresses
    );

    // Create SocketAddrInformation for woodstock resolver
    let information = SocketAddrInformation {
        source: SocketAddrInformationSource::DIRECT, // Direct registration from client
        refresh_date: chrono::Local::now().timestamp(),
        hostname: hostname.clone(),
        port: body.port,
        version: body.version.clone(),
        addresses,
        is_online: true,
    };

    // Register the information with the resolver
    state
        .resolver
        .register_service(&information)
        .await
        .map_err(|e| {
            ApiError::InternalServerError(format!("Failed to register client information: {}", e))
        })?;

    info!(
        "Successfully registered client '{}' with {} addresses on port {}",
        hostname,
        information.addresses.len(),
        body.port
    );

    // Return empty response (matches NestJS behavior)
    Ok(Json(serde_json::json!({})))
}

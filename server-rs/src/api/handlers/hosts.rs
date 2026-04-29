//! Host management handlers

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::Response,
    Json,
};
use eyre::Result;
use serde::Deserialize;
use std::io::Cursor;
use tokio::io::duplex;
use tokio_util::io::ReaderStream;
use tracing::Instrument;
// zip crate is used in tests; runtime uses async_zip
use async_zip::tokio::write::ZipFileWriter as AsyncZipWriter;
use async_zip::{AttributeCompatibility, Compression, ZipEntryBuilder};

use crate::api::{
    dto::{ClientType, HostConfiguration, HostInformation},
    ApiError, ApiResult, ApiServerState,
};

// Plus de writer custom: on utilise un duplex tokio (backpressure) et async-zip.

/// Query parameters for the client download endpoint
#[derive(Debug, Deserialize)]
pub struct ClientDownloadQuery {
    #[serde(default)]
    pub client: ClientType,
}

/// URL mapping for client binaries download (matching TypeScript implementation)
fn zip_url_mapping(client_type: &ClientType) -> Option<&'static str> {
    match client_type {
        ClientType::Windows => Some("binaries-windows-x86_64-pc-windows-msvc.zip"),
        ClientType::Linux => Some("binaries-linux-x86_64-unknown-linux-gnu.zip"),
        ClientType::LinuxLite => Some("binaries-linux-lite-x86_64-unknown-linux-gnu.zip"),
        ClientType::LinuxDeb | ClientType::None => None,
    }
}

/// Get client daemon filename (matching TypeScript implementation)
fn client_daemon_filename(windows: bool) -> &'static str {
    if windows {
        "ws_client_daemon.exe"
    } else {
        "ws_client_daemon"
    }
}

/// Find version from package.json (simplified version for Rust)
fn find_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Download client binary from GitHub releases
async fn find_client(client_type: &ClientType, version: &str) -> Result<Vec<u8>> {
    let zip_name = zip_url_mapping(client_type)
        .ok_or_else(|| eyre::eyre!("Unsupported client type: {:?}", client_type))?;
    let client_url = format!(
        "https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/releases/download/v{}/{}",
        version, zip_name
    );

    tracing::info!("Downloading client from {}", client_url);

    // Download the ZIP file
    let response = reqwest::get(&client_url).await?;
    if !response.status().is_success() {
        return Err(eyre::eyre!(
            "Failed to download client: HTTP {}",
            response.status()
        ));
    }

    let zip_data = response.bytes().await?;

    // Extract the daemon executable from the ZIP
    let cursor = Cursor::new(zip_data.as_ref());
    let mut archive = zip::read::ZipArchive::new(cursor)?;

    let daemon_filename = client_daemon_filename(matches!(client_type, ClientType::Windows));

    // Find the daemon file in the ZIP
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.name() == daemon_filename {
            let mut buffer = Vec::new();
            std::io::copy(&mut file, &mut buffer)?;
            return Ok(buffer);
        }
    }

    Err(eyre::eyre!("File {} not found in the ZIP", daemon_filename))
}

/// Get all hosts
#[utoipa::path(
    get,
    path = "/api/hosts",
    tag = "hosts",
    responses(
        (status = 200, description = "List of all hosts", body = [HostInformation])
    )
)]
pub async fn get_hosts(
    State(state): State<ApiServerState>,
) -> ApiResult<Json<Vec<HostInformation>>> {
    let host_names = state.hosts_service.list_hosts().await.map_err(|e| {
        tracing::error!("Failed to list hosts: {}", e);
        ApiError::InternalServerError("Failed to retrieve hosts list".to_string())
    })?;

    let mut hosts_info = Vec::new();
    for host_name in host_names {
        match state.hosts_service.get_host_information(&host_name).await {
            Ok(host_info) => hosts_info.push(host_info),
            Err(e) => {
                tracing::warn!("Failed to get information for host '{}': {}", host_name, e);
                // Continue with other hosts, don't fail the entire request
            }
        }
    }

    Ok(Json(hosts_info))
}

/// Get specific host configuration
#[utoipa::path(
    get,
    path = "/api/hosts/{name}",
    tag = "hosts",
    params(
        ("name" = String, Path, description = "Host name")
    ),
    responses(
        (status = 200, description = "Host configuration", body = HostConfiguration),
        (status = 404, description = "Host not found")
    )
)]
pub async fn get_host(
    State(state): State<ApiServerState>,
    Path(name): Path<String>,
) -> ApiResult<Json<HostConfiguration>> {
    match state
        .hosts_service
        .get_public_host_configuration(&name)
        .await
    {
        Ok(host_config) => Ok(Json(host_config)),
        Err(e) => {
            tracing::error!("Failed to get host configuration for '{}': {}", name, e);
            // Check if it's a "not found" error or a more general error
            let error_msg = e.to_string();
            if error_msg.contains("not found") || error_msg.contains("No such host") {
                Err(ApiError::NotFound(format!("Host '{}' not found", name)))
            } else {
                Err(ApiError::InternalServerError(
                    "Failed to retrieve host configuration".to_string(),
                ))
            }
        }
    }
}

/// Download client configuration for host
#[utoipa::path(
    get,
    path = "/api/hosts/{name}/client",
    tag = "hosts",
    params(
        ("name" = String, Path, description = "Host name"),
        ("client" = Option<ClientType>, Query, description = "Client type for binary download")
    ),
    responses(
        (status = 200, description = "Client configuration archive", content_type = "application/zip"),
        (status = 404, description = "Host not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_host_client_download(
    State(state): State<ApiServerState>,
    Path(name): Path<String>,
    Query(query): Query<ClientDownloadQuery>,
) -> ApiResult<Response> {
    // First, verify that the host exists
    let host_information = match state
        .hosts_service
        .get_private_host_configuration(&name)
        .await
    {
        Ok(info) => info,
        Err(e) => {
            tracing::error!("Failed to get host configuration for '{}': {}", name, e);
            return Err(ApiError::NotFound(format!("Host '{}' not found", name)));
        }
    };

    // Generate all required certificates for the host
    if let Err(e) = state
        .certificate_service
        .generate_host_certificate(&name)
        .await
    {
        tracing::error!("Failed to generate certificates for host '{}': {}", name, e);
        return Err(ApiError::InternalServerError(
            "Failed to generate certificates".to_string(),
        ));
    }

    // Crée un pipe en mémoire borné; la backpressure limite la mémoire si le réseau est lent
    let (mut writer, reader) = duplex(64 * 1024);

    // Données clonées pour la tâche d'écriture
    let state_clone = state.clone();
    let name_clone = name.clone();
    let password_clone = host_information.password.clone();
    let client_type_clone = query.client.clone();

    // Tâche d'écriture ZIP full-async avec async-zip
    tokio::spawn(
        async move {
            let result: Result<(), eyre::Report> = async {
                let cert_path = state_clone.certificate_service.certificate_path();
                let mut zip = AsyncZipWriter::with_tokio(&mut writer);

                // Certificats (même liste que l'impl TypeScript)
                let server_pem = format!("{}_server.pem", name_clone);
                let server_key = format!("{}_server.key", name_clone);
                let https_pem = format!("{}_https.pem", name_clone);
                let https_key = format!("{}_https.key", name_clone);

                let cert_files = vec![
                    ("rootCA.pem".to_string(), "rootCA.pem"),
                    ("public_key.pem".to_string(), "public_key.pem"),
                    (server_pem.clone(), server_pem.as_str()),
                    (server_key.clone(), server_key.as_str()),
                    (https_pem.clone(), https_pem.as_str()),
                    (https_key.clone(), https_key.as_str()),
                ];

                for (source_filename, zip_filename) in cert_files {
                    let file_path = cert_path.join(&source_filename);
                    match tokio::fs::read(&file_path).await {
                        Ok(content) => {
                            let builder =
                                ZipEntryBuilder::new(zip_filename.into(), Compression::Deflate);
                            zip.write_entry_whole(builder, &content).await?;
                            tracing::debug!(
                                "Added {} to streaming ZIP for host {}",
                                zip_filename,
                                name_clone
                            );
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Certificate file {} not found for host {} ({}), skipping",
                                source_filename,
                                name_clone,
                                e
                            );
                        }
                    }
                }

                // config.yaml
                let config_yaml = format!(
                    r#"hostname: {}
password: {}
resolution_mode: Direct
server: https://localhost:8443
"#,
                    name_clone, password_clone
                );
                let builder = ZipEntryBuilder::new("config.yaml".into(), Compression::Deflate);
                zip.write_entry_whole(builder, config_yaml.as_bytes())
                    .await?;

                // Binaire client si demandé
                if !matches!(client_type_clone, ClientType::None | ClientType::LinuxDeb) {
                    let version = find_version();
                    match find_client(&client_type_clone, version).await {
                        Ok(client_binary) => {
                            let is_windows = matches!(client_type_clone, ClientType::Windows);
                            let filename = client_daemon_filename(is_windows);
                            tracing::info!(
                                "Adding client binary {} to streaming ZIP for host {}",
                                filename,
                                name_clone
                            );
                            // Pour compatibilité max:
                            // - Windows: attributs DOS (ignore les permissions Unix)
                            // - Linux: attributs Unix + permissions exécutable
                            let builder = if is_windows {
                                // Ne force pas de compatibilité spécifique côté Windows; laisse par défaut
                                ZipEntryBuilder::new(filename.into(), Compression::Deflate)
                            } else {
                                ZipEntryBuilder::new(filename.into(), Compression::Deflate)
                                    .attribute_compatibility(AttributeCompatibility::Unix)
                                    .unix_permissions(0o755)
                            };
                            zip.write_entry_whole(builder, &client_binary).await?;
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to download client binary for host {} ({}), skipping",
                                name_clone,
                                e
                            );
                        }
                    }
                }

                // Finalise
                zip.close().await?;
                Ok(())
            }
            .await;

            if let Err(e) = result {
                tracing::error!(
                    "Failed to create streaming ZIP for host '{}': {}",
                    name_clone,
                    e
                );
                // La fermeture du writer signale la fin au lecteur; rien d'autre à faire
            }
            // writer est drop ici -> fin du flux côté lecteur
        }
        .in_current_span(),
    );

    // Reader -> Body via ReaderStream (backpressure gérée par le duplex)
    let body = Body::from_stream(ReaderStream::new(reader));

    // Retourne la réponse streaming avec les bons headers
    let content_disposition = format!("attachment; filename=\"{}_client.zip\"", name);
    let resp = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/zip")
        .header(header::CONTENT_DISPOSITION, content_disposition)
        .body(body)
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(resp)
}

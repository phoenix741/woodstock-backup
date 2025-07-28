//! Backup management handlers

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::Response,
    Json,
};
use futures::StreamExt;
use std::convert::Infallible;
use tracing::{debug, Instrument};
use uuid::Uuid;

use crate::api::{dto::Backup, ApiError, ApiResult, ApiServerState};

/// List backups for a host
#[utoipa::path(
    get,
    path = "/api/hosts/{name}/backups",
    tag = "backups",
    params(("name" = String, Path, description = "Host name")),
    responses((status = 200, description = "List of backups", body = [Backup]), (status = 404, description = "Host not found"))
)]
pub async fn list_host_backups(
    State(state): State<ApiServerState>,
    Path(name): Path<String>,
) -> ApiResult<Json<Vec<Backup>>> {
    // Validate host exists
    let hosts = state
        .hosts_service
        .list_hosts()
        .await
        .map_err(|_| ApiError::InternalServerError("Failed to list hosts".into()))?;
    if !hosts.contains(&name) {
        return Err(ApiError::NotFound(format!(
            "Can't find the host with the name {}",
            name
        )));
    }

    let backups = state
        .backups_service
        .get_backups(&name)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get backups for {}: {}", name, e);
            ApiError::InternalServerError("Failed to retrieve backups".into())
        })?;
    Ok(Json(backups))
}

/// Create a new backup for a host (enqueue job)
#[utoipa::path(
    post,
    path = "/api/hosts/{name}/backups",
    tag = "backups",
    params(("name" = String, Path, description = "Host name")),
    responses(
        (status = 201, description = "Backup job enqueued"),
        (status = 202, description = "Backup already scheduled recently (idempotent)"),
        (status = 404, description = "Host not found")
    )
)]
pub async fn create_host_backup(
    State(state): State<ApiServerState>,
    Path(name): Path<String>,
) -> ApiResult<StatusCode> {
    let hosts = state
        .hosts_service
        .list_hosts()
        .await
        .map_err(|_| ApiError::InternalServerError("Failed to list hosts".into()))?;
    if !hosts.contains(&name) {
        return Err(ApiError::NotFound(format!(
            "Can't find the host with the name {}",
            name
        )));
    }

    // Enqueue backup job with uniqueness (like GraphQL create_backup)
    let mut producers = state.producers.lock().await;
    match producers
        .enqueue_backup_unique(&name, true)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
    {
        Some(job_id) => {
            tracing::info!("Backup job enqueued for host {}: {}", name, job_id);
            Ok(StatusCode::CREATED)
        }
        None => {
            // Unicité: un job équivalent récent existe déjà
            tracing::info!("Backup already scheduled recently for host {}", name);
            Ok(StatusCode::ACCEPTED)
        }
    }
}

/// Remove a backup for a host (enqueue job)
#[utoipa::path(
    delete,
    path = "/api/hosts/{name}/backups/{id}",
    tag = "backups",
    params(
        ("name" = String, Path, description = "Host name"),
        ("id" = String, Path, description = "Backup UUID")
    ),
    responses((status = 200, description = "Delete a backup"), (status = 404, description = "Host not found"))
)]
pub async fn remove_host_backup(
    State(state): State<ApiServerState>,
    Path((name, id)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    let hosts = state
        .hosts_service
        .list_hosts()
        .await
        .map_err(|_| ApiError::InternalServerError("Failed to list hosts".into()))?;
    if !hosts.contains(&name) {
        return Err(ApiError::NotFound(format!(
            "Can't find the host with the name {}",
            name
        )));
    }

    let backup_id = Uuid::parse_str(&id).map_err(|e| ApiError::BadRequest(e.to_string()))?;

    // Validate backup exists
    let Some(backup) = state
        .backups_service
        .get_backup(&name, backup_id)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
    else {
        return Err(ApiError::NotFound(format!(
            "Can't find the backup {} for the host {}",
            id, name
        )));
    };

    let mut producers = state.producers.lock().await;
    let job_id = producers
        .enqueue_remove(&name, backup_id, backup.number)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    tracing::info!(
        "Remove backup job enqueued for host {} number {}: {}",
        name,
        backup.number,
        job_id
    );
    Ok(StatusCode::OK)
}

/// Get the application log of a backup (static or tail via query tailable=true)
#[utoipa::path(
    get,
    path = "/api/hosts/{name}/backups/{id}/log/backup.log",
    tag = "backups",
    params(
        ("name" = String, Path, description = "Host name"),
        ("id" = String, Path, description = "Backup UUID"),
        ("tailable" = bool, Query, description = "Tail the log")
    ),
    responses((status = 200, description = "Backup log", content_type = "text/plain"))
)]
pub async fn get_backup_log(
    State(state): State<ApiServerState>,
    Path((name, id)): Path<(String, String)>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> ApiResult<Response> {
    let tailable = params
        .get("tailable")
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(false);

    let backup_id = Uuid::parse_str(&id).map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let Some(_backup) = state
        .backups_service
        .get_backup(&name, backup_id)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
    else {
        return Err(ApiError::NotFound(format!(
            "Backup {} not found for {}",
            id, name
        )));
    };

    let filename = state
        .backups_service
        .get_log_directory(&name, backup_id)
        .join("backup.log");

    stream_or_read_log(&filename, tailable).await
}

/// Get the error log of a backup (static or tail)
#[utoipa::path(
    get,
    path = "/api/hosts/{name}/backups/{id}/log/backup.error.log",
    tag = "backups",
    params(
        ("name" = String, Path, description = "Host name"),
        ("id" = String, Path, description = "Backup UUID"),
        ("tailable" = bool, Query, description = "Tail the log")
    ),
    responses((status = 200, description = "Backup error log", content_type = "text/plain"))
)]
pub async fn get_backup_error_log(
    State(state): State<ApiServerState>,
    Path((name, id)): Path<(String, String)>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> ApiResult<Response> {
    let tailable = params
        .get("tailable")
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(false);

    let backup_id = Uuid::parse_str(&id).map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let Some(_backup) = state
        .backups_service
        .get_backup(&name, backup_id)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
    else {
        return Err(ApiError::NotFound(format!(
            "Backup {} not found for {}",
            id, name
        )));
    };

    let filename = state
        .backups_service
        .get_backup_destination_directory(&name, backup_id)
        .join("backup-error.log");

    stream_or_read_log(&filename, tailable).await
}

/// Get the xferLog for a backup/share
#[utoipa::path(
    get,
    path = "/api/hosts/{name}/backups/{id}/xferLog/{share}",
    tag = "backups",
    params(
        ("name" = String, Path, description = "Host name"),
        ("id" = String, Path, description = "Backup UUID"),
        ("share" = String, Path, description = "Share name")
    ),
    responses((status = 200, description = "Share transfer log", content_type = "text/plain"))
)]
pub async fn get_backup_xfer_log(
    State(state): State<ApiServerState>,
    Path((name, id, share)): Path<(String, String, String)>,
) -> ApiResult<Response> {
    // Stream log lines from manifest journal; use a channel to avoid lifetime issues
    use tokio_stream::wrappers::ReceiverStream;
    debug!("Getting xfer log for backup {id} on host {name} (share: {share})");
    let backup_id = Uuid::parse_str(&id).map_err(|e| ApiError::BadRequest(e.to_string()))?;
    let Some(_backup) = state
        .backups_service
        .get_backup(&name, backup_id)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
    else {
        return Err(ApiError::NotFound(format!(
            "Backup {} not found for {}",
            id, name
        )));
    };
    let manifest = state.backups_service.get_manifest(&name, backup_id, &share);
    debug!("Manifest for backup {id} on host {name} (share: {share}): {manifest:?}");
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<axum::body::Bytes, Infallible>>(64);

    tokio::spawn(
        async move {
            let mut stream = Box::pin(manifest.read_log_entries());
            while let Some(entry) = stream.next().await {
                let line = entry.to_log();
                if tx
                    .send(Ok(axum::body::Bytes::from(format!("{line}\n"))))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        }
        .in_current_span(),
    );

    let resp = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from_stream(ReceiverStream::new(rx)))
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    Ok(resp)
}

async fn stream_or_read_log(path: &std::path::Path, tailable: bool) -> ApiResult<Response> {
    if tailable {
        // Streaming continu comme 'tail -f' :
        // 1) envoyer tout le contenu actuel depuis le début
        // 2) surveiller les ajouts et pousser les nouveaux octets
        use axum::body::Bytes;
        use tokio::io::{AsyncReadExt, AsyncSeekExt};
        use tokio_stream::wrappers::ReceiverStream;

        let (tx, rx) = tokio::sync::mpsc::channel::<Result<Bytes, std::convert::Infallible>>(64);
        let path_clone = path.to_path_buf();

        tokio::spawn(
            async move {
                let res: eyre::Result<()> = async {
                    use std::time::Duration;
                    use tokio::fs::File;
                    use tokio::io::SeekFrom;

                    // Ouvrir le fichier et envoyer son contenu existant
                    let mut file = File::open(&path_clone).await?;
                    let mut pos: u64 = 0;

                    // Lire le contenu initial par chunks
                    let mut buf = vec![0u8; 8 * 1024];
                    loop {
                        let n = file.read(&mut buf).await?;
                        if n == 0 {
                            break;
                        }
                        if tx
                            .send(Ok(Bytes::copy_from_slice(&buf[..n])))
                            .await
                            .is_err()
                        {
                            return Ok(()); // Client déconnecté
                        }
                        pos += n as u64;
                    }

                    // Boucle de surveillance des ajouts
                    let mut interval = tokio::time::interval(Duration::from_millis(500));
                    loop {
                        interval.tick().await;

                        // Si le fichier a été tronqué ou remplacé, réouvrir et repartir du début
                        let meta = match tokio::fs::metadata(&path_clone).await {
                            Ok(m) => m,
                            Err(e) => {
                                // Fichier supprimé ou inaccessible : arrêter proprement
                                tracing::warn!(
                                    "Log file {:?} is not accessible anymore: {}",
                                    path_clone,
                                    e
                                );
                                break;
                            }
                        };
                        let size = meta.len();

                        if size < pos {
                            // Troncature/rotation : réouvrir et repartir de 0
                            file = File::open(&path_clone).await?;
                            pos = 0;
                        }

                        if size > pos {
                            file.seek(SeekFrom::Start(pos)).await?;

                            let mut remaining = size - pos;
                            while remaining > 0 {
                                let to_read = std::cmp::min(remaining, buf.len() as u64) as usize;
                                let n = file.read(&mut buf[..to_read]).await?;
                                if n == 0 {
                                    break;
                                }
                                if tx
                                    .send(Ok(Bytes::copy_from_slice(&buf[..n])))
                                    .await
                                    .is_err()
                                {
                                    return Ok(()); // Client déconnecté
                                }
                                pos += n as u64;
                                remaining -= n as u64;
                            }
                        }
                    }

                    Ok(())
                }
                .await;

                if let Err(e) = res {
                    tracing::error!("Failed to tail log file {:?}: {}", path_clone, e);
                }
            }
            .in_current_span(),
        );

        let stream = ReceiverStream::new(rx);
        let body = Body::from_stream(stream);

        let resp = Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .header(header::CACHE_CONTROL, "no-cache")
            .header("X-Accel-Buffering", "no") // Nginx: disable buffering pour streaming temps réel
            .body(body)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        Ok(resp)
    } else {
        // Lecture statique complète du fichier
        let content = tokio::fs::read(path)
            .await
            .map_err(|e| ApiError::NotFound(format!("Log file not found: {}", e)))?;

        let resp = Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(Body::from(content))
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        Ok(resp)
    }
}

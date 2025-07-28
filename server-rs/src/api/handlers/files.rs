//! File management handlers

use async_zip::tokio::write::ZipFileWriter as AsyncZipWriter;
use async_zip::{Compression, StringEncoding, ZipEntryBuilder, ZipString};
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::Response,
    Json,
};
use futures_util::io::AsyncWriteExt as FuturesAsyncWriteExt;
use tokio::io::duplex;
use tokio::io::AsyncReadExt;
use tokio_util::io::ReaderStream;
use tracing::{debug, Instrument};
// removed unused tracing_subscriber::field::debug import
use woodstock::utils::path::{path_to_vec, unmangle, unmangle_buffer, unmangle_path}; // for read_to_end on AsyncRead/AsyncBufRead
                                                                                     // PathManifest trait not required; we read last_modified from stats

use crate::api::{dto::FileDescription, ApiError, ApiResult, ApiServerState};

#[derive(Debug, serde::Deserialize)]
pub struct FilesQuery {
    #[serde(rename = "sharePath")]
    pub share_path: Option<String>,
    pub path: Option<String>,
}

/// List shares or files depending on query params (matches NestJS behavior)
#[utoipa::path(
    get,
    path = "/api/hosts/{name}/backups/{id}/files",
    tag = "files",
    params(
        ("name" = String, Path, description = "Host name"),
        ("id" = String, Path, description = "Backup UUID"),
        ("sharePath" = Option<String>, Query, description = "Share path"),
        ("path" = Option<String>, Query, description = "Sub path")
    ),
    responses((status = 200, description = "List of files", body = [FileDescription]))
)]
pub async fn list_files_or_shares(
    State(state): State<ApiServerState>,
    Path((name, id)): Path<(String, String)>,
    Query(q): Query<FilesQuery>,
) -> ApiResult<Json<Vec<FileDescription>>> {
    let backup_id = uuid::Uuid::parse_str(&id).map_err(|e| ApiError::BadRequest(e.to_string()))?;
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

    let mut files = (*state.files_service).clone();
    if q.share_path.is_none() {
        let shares = files.list_shares(&name, backup_id).await?;

        Ok(Json(shares))
    } else {
        // SAFETY: share_path is Some here — the is_none() check above returned the early branch
        let share = q.share_path.unwrap();
        let subpath = q.path.unwrap_or_else(|| "/".into());
        let subpath = unmangle_buffer(&subpath);

        let files = files.list_files(&name, backup_id, &share, &subpath).await?;

        Ok(Json(files))
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct DownloadQuery {
    #[serde(rename = "sharePath")]
    pub share_path: String,
    pub path: String,
}

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

/// Download an archive (ZIP) of selected files
#[utoipa::path(
    get,
    path = "/api/hosts/{name}/backups/{id}/files/download",
    tag = "files",
    params(
        ("name" = String, Path, description = "Host name"),
        ("id" = String, Path, description = "Backup UUID"),
        ("sharePath" = String, Query, description = "Share path"),
        ("path" = String, Query, description = "Sub path")
    ),
    responses((status = 200, description = "Archive file", content_type = "application/zip"))
)]
pub async fn download_archive(
    State(state): State<ApiServerState>,
    Path((name, id)): Path<(String, String)>,
    Query(q): Query<DownloadQuery>,
    _headers: axum::http::HeaderMap,
) -> ApiResult<Response> {
    // viewer créé dans la tâche d'écriture pour limiter l'usage mémoire
    let base_in_archive = sanitize_filename(&format!("{}_{}", q.share_path, q.path));
    let base_in_archive_header = base_in_archive.clone();

    let backup_id = uuid::Uuid::parse_str(&id).map_err(|e| ApiError::BadRequest(e.to_string()))?;

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

    // Prepare streaming ZIP via duplex
    let (mut writer, reader) = duplex(64 * 1024);
    let view = woodstock::view::WoodstockView::new(
        state.config.clone(),
        state.hosts.clone(),
        state.backups.clone(),
    );

    let name_c = name.clone();
    let share_c = unmangle(&q.share_path);
    let subpath_c = unmangle_path(&q.path);

    let pool_path = state.config.path.pool_path.clone();

    tokio::spawn(
        async move {
            let res: eyre::Result<()> = async {
                let mut zip = AsyncZipWriter::with_tokio(&mut writer);
                let mut view = view;

                debug!("Base path for archive: {name_c}, {backup_id}, {subpath_c:?}");

                let all = view
                    .list_all_files(&name_c, backup_id, &share_c, &subpath_c)
                    .await?;

                for manifest in all {
                    debug!("Adding to archive: {:?}", manifest.path());

                    if !manifest.is_regular_file() {
                        continue;
                    }
                    debug!("Added");

                    let manifest_path = manifest.path();
                    let rel = path_to_vec(
                        manifest_path
                            .strip_prefix(&subpath_c)
                            .unwrap_or(manifest_path.as_path()),
                    );

                    let reader = manifest.open_from_pool(&pool_path);

                    // Stream file content directly into the ZIP entry to avoid buffering large files in memory
                    let builder = ZipEntryBuilder::new(
                        ZipString::new(rel, StringEncoding::Utf8),
                        Compression::Deflate,
                    );

                    let mut entry = zip.write_entry_stream(builder).await?;

                    // Stream from potentially non-Unpin reader into futures::AsyncWrite entry
                    let mut buf = vec![0u8; 64 * 1024];
                    tokio::pin!(reader);
                    loop {
                        let n = reader.read(&mut buf).await?;
                        if n == 0 {
                            break;
                        }
                        FuturesAsyncWriteExt::write_all(&mut entry, &buf[..n]).await?;
                    }
                    entry.close().await?;
                }

                zip.close().await?;
                Ok(())
            }
            .await;
            if let Err(e) = res {
                tracing::error!("Failed to create ZIP archive: {}", e);
            }
        }
        .in_current_span(),
    );

    // Build response
    let body = Body::from_stream(ReaderStream::new(reader));
    let content_disposition = format!("attachment; filename=\"{}.zip\"", base_in_archive_header);
    let resp = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/zip")
        .header(header::CONTENT_DISPOSITION, content_disposition)
        .body(body)
        .map_err(|e| crate::api::ApiError::InternalServerError(e.to_string()))?;
    Ok(resp)
}

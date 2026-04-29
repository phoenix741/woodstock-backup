//! Server management handlers

use axum::{extract::State, http::StatusCode, response::Response};

use crate::api::{ApiError, ApiResult, ApiServerState};

/// Clear server cache
#[utoipa::path(
    post,
    path = "/api/server/cache/clear",
    tag = "server",
    responses(
        (status = 200, description = "Cache cleared successfully")
    )
)]
pub async fn clear_cache(State(state): State<ApiServerState>) -> ApiResult<StatusCode> {
    state
        .server_service
        .clear_cache()
        .await
        .map_err(|_| ApiError::InternalServerError("Failed to clear cache".to_string()))?;
    Ok(StatusCode::OK)
}

/// Get application log
#[utoipa::path(
    get,
    path = "/api/server/logs/application.log",
    tag = "server",
    responses(
        (status = 200, description = "Application log content", content_type = "text/plain")
    )
)]
pub async fn get_application_log(State(state): State<ApiServerState>) -> ApiResult<Response> {
    let log_content = state
        .server_service
        .read_log_file("application.log")
        .await
        .map_err(|_| ApiError::InternalServerError("Failed to read application log".to_string()))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/plain")
        .body(log_content.into())
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?)
}

/// Get exceptions log
#[utoipa::path(
    get,
    path = "/api/server/logs/exceptions.log",
    tag = "server",
    responses(
        (status = 200, description = "Exceptions log content", content_type = "text/plain")
    )
)]
pub async fn get_exceptions_log(State(state): State<ApiServerState>) -> ApiResult<Response> {
    let log_content = state
        .server_service
        .read_log_file("exceptions.log")
        .await
        .map_err(|_| ApiError::InternalServerError("Failed to read exceptions log".to_string()))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/plain")
        .body(log_content.into())
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?)
}

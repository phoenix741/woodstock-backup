//! Prometheus metrics handler

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::Response,
};

use crate::api::{ApiResult, ApiServerState};

/// Get Prometheus metrics
#[utoipa::path(
    get,
    path = "/metrics",
    tag = "metrics",
    responses(
        (status = 200, description = "Prometheus metrics in text format", content_type = "text/plain")
    )
)]
pub async fn get_metrics(State(state): State<ApiServerState>) -> ApiResult<Response> {
    // Get metrics in Prometheus format (automatically updates cache if needed)
    let metrics = state.metrics_service.get_metrics().await.map_err(|e| {
        tracing::error!("Failed to gather metrics: {}", e);
        crate::api::ApiError::InternalServerError("Failed to gather metrics".to_string())
    })?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )
        .body(metrics.into())
        .map_err(|e| crate::api::ApiError::InternalServerError(e.to_string()))?)
}

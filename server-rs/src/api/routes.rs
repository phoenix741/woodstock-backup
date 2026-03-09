//! Route definitions for the public API server
//!
//! Implements the /api prefix routing (except /metrics) to match NestJS structure

use axum::{
    routing::{delete, get, post},
    Router,
};
use std::path::PathBuf;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::{debug, info};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::{handlers, ApiServerState};
use crate::graphql::{build_schema, graphql_router};

/// OpenAPI documentation definition
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::metrics::get_metrics,
        handlers::hosts::get_hosts,
        handlers::hosts::get_host,
        handlers::hosts::get_host_client_download,
        handlers::backups::list_host_backups,
        handlers::backups::create_host_backup,
        handlers::backups::remove_host_backup,
        handlers::backups::get_backup_log,
        handlers::backups::get_backup_error_log,
        handlers::backups::get_backup_xfer_log,
        handlers::files::list_files_or_shares,
        handlers::files::download_archive,
        handlers::server::clear_cache,
    ),
    components(
        schemas(
            crate::api::dto::Backup,
            crate::api::dto::HostInformation,
            crate::api::dto::HostConfiguration,
            crate::api::dto::ClientType,
            crate::api::dto::HostConfigOperation,
            crate::api::dto::BackupOperation,
            crate::api::dto::ExecuteCommandOperation,
            crate::api::dto::BackupTaskShare,
            crate::api::dto::Schedule,
            crate::api::dto::ScheduledBackupToKeep,
            crate::api::dto::FileDescription,
            crate::api::dto::BackupStatusDto,
            crate::api::dto::Statistics,
            crate::api::dto::CreateBackupRequest,
            crate::api::error::ApiErrorResponse,
        )
    ),
    tags(
        (name = "hosts", description = "Host management endpoints"),
        (name = "backups", description = "Backup management endpoints"),
        (name = "files", description = "File management and download endpoints"),
        (name = "server", description = "Server management endpoints"),
        (name = "metrics", description = "Prometheus metrics endpoint"),
    )
)]
pub struct ApiDoc;

/// Create the main router for the API server
pub fn create_router(state: ApiServerState, static_path: Option<String>) -> Router {
    debug!("Create REST routes");
    // Create the /api prefixed routes (excludes /metrics)
    // Garder le type d'état manquant explicite pour l'inférence
    let api_routes: Router<ApiServerState> = Router::new()
        // Hosts endpoints
        .route("/hosts", get(handlers::hosts::get_hosts))
        .route("/hosts/{name}", get(handlers::hosts::get_host))
        .route(
            "/hosts/{name}/client",
            get(handlers::hosts::get_host_client_download),
        )
        // Backups endpoints (scoped under host like NestJS)
        .route(
            "/hosts/{name}/backups",
            get(handlers::backups::list_host_backups).post(handlers::backups::create_host_backup),
        )
        .route(
            "/hosts/{name}/backups/{number}",
            delete(handlers::backups::remove_host_backup),
        )
        .route(
            "/hosts/{name}/backups/{number}/log/backup.log",
            get(handlers::backups::get_backup_log),
        )
        .route(
            "/hosts/{name}/backups/{number}/log/backup.error.log",
            get(handlers::backups::get_backup_error_log),
        )
        .route(
            "/hosts/{name}/backups/{number}/xferLog/{share}",
            get(handlers::backups::get_backup_xfer_log),
        )
        // Files endpoints under backups
        .route(
            "/hosts/{name}/backups/{number}/files",
            get(handlers::files::list_files_or_shares),
        )
        .route(
            "/hosts/{name}/backups/{number}/files/download",
            get(handlers::files::download_archive),
        )
        // Server management endpoints
        .route("/server/cache/clear", post(handlers::server::clear_cache))
        .route(
            "/server/logs/application.log",
            get(handlers::server::get_application_log),
        )
        .route(
            "/server/logs/exceptions.log",
            get(handlers::server::get_exceptions_log),
        );

    debug!("Create Swagger routes");
    // Build Swagger UI router
    let swagger_ui = SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", ApiDoc::openapi());

    debug!("Create Graphql routes");
    // Main router with /api prefix and /metrics exception
    let schema = build_schema(state.clone());
    let gql_router = graphql_router(schema);

    debug!("Create application");
    let app: Router<ApiServerState> = Router::new()
        // Metrics endpoint (no /api prefix as per NestJS configuration)
        .route("/metrics", get(handlers::metrics::get_metrics))
        // All other routes with /api prefix
        .nest("/api", api_routes)
        // GraphQL endpoints at /graphql and /graphql/ws
        .merge(gql_router)
        // Swagger UI for API documentation (compat with current utoipa-swagger-ui)
        .merge(swagger_ui)
        // Add tracing middleware
        .layer(TraceLayer::new_for_http());

    let mut final_app = app.with_state(state);

    if let Some(path_str) = static_path {
        let path = PathBuf::from(path_str);
        if path.exists() {
            info!("Serving static files from {:?}", path);
            // Serve files from directory, and fallback to index.html for SPA routing
            let serve_dir =
                ServeDir::new(&path).not_found_service(ServeFile::new(path.join("index.html")));

            final_app = final_app.fallback_service(serve_dir);
        } else {
            tracing::warn!(
                "Static path {:?} does not exist, skipping static file serving",
                path
            );
        }
    }

    final_app
}

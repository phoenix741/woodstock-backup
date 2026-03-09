//! Application state for the public API server

use super::services::{
    BackupsService, CertificateService, FilesService, HostsService, MetricsService, QueueService,
    ServerService,
};
use crate::{
    jobs::{
        producers::Producers,
        progress::{ProgressPublisher, ProgressReader},
    },
    shared_state::SharedState,
};
use eyre::Result;
use std::sync::Arc;
use woodstock::config::Configuration;

/// Shared application state for the API server.
///
/// Embeds a [`SharedState`] (infrastructure shared with the job worker) plus
/// service objects that are specific to the REST / GraphQL API. Implements
/// [`std::ops::Deref`] so shared fields (`config`, `hosts`, …) are directly
/// accessible on the state object.
#[derive(Clone)]
pub struct ApiServerState {
    /// Shared infrastructure (config, scheduler, hosts, backups, resolver, job_utility).
    pub shared: SharedState,

    /// Hosts service for managing host configurations
    pub hosts_service: Arc<HostsService>,

    /// Backups service for managing backup operations
    pub backups_service: Arc<BackupsService>,

    /// Files service for managing backup files
    pub files_service: Arc<FilesService>,

    /// Queue service for managing job queue
    pub queue_service: Arc<QueueService>,

    /// Metrics service for Prometheus metrics
    pub metrics_service: Arc<MetricsService>,

    /// Server management service
    pub server_service: Arc<ServerService>,

    /// Certificate service for X.509 certificate generation
    pub certificate_service: Arc<CertificateService>,

    /// Shared job producers (Apalis)
    pub producers: Arc<tokio::sync::Mutex<Producers>>,

    /// Service to read progression
    pub progress_reader: Arc<ProgressReader>,

    /// Raw Redis client — used to open pubsub connections on demand
    /// (e.g., `backupUpdated` subscription).
    pub redis_client: redis::Client,
}

impl std::ops::Deref for ApiServerState {
    type Target = SharedState;

    fn deref(&self) -> &Self::Target {
        &self.shared
    }
}

impl ApiServerState {
    /// Create new API server state
    pub async fn new(config: Arc<Configuration>) -> Result<Self> {
        // Initialize metrics
        super::services::metrics::init_metrics()?;

        let (shared, redis_client) = SharedState::new(config.clone()).await?;

        // Create business logic services with woodstock-rs integration
        let backups_service = Arc::new(BackupsService::new(
            shared.backups.clone(),
            shared.job_utility.clone(),
        ));
        let hosts_service = Arc::new(HostsService::new(
            shared.hosts.clone(),
            backups_service.clone(),
        ));
        let files_service = Arc::new(FilesService::new(
            config.clone(),
            shared.hosts.clone(),
            shared.backups.clone(),
            backups_service.clone(),
        ));
        let queue_service = Arc::new(QueueService::new(config.clone()).await?);
        let metrics_service = Arc::new(MetricsService::new());

        // Create certificate service
        let certificate_service = Arc::new(CertificateService::new(config.clone()));

        // Create server management service
        let server_service = Arc::new(ServerService::new(
            hosts_service.clone(),
            config.path.logs_path.clone(),
        ));

        let progress_publisher = ProgressPublisher::new(redis_client.clone()).await?;

        // Build shared Producers from storages
        let producers = Arc::new(tokio::sync::Mutex::new(Producers::new(
            shared.hosts.clone(),
            shared.backups.clone(),
            queue_service.schedule_storage(),
            queue_service.backup_storage(),
            queue_service.interactive_storage(),
            queue_service.maintenance_storage(),
            progress_publisher,
            redis_client.clone(),
        )));

        // Build progress reader
        let progress_reader = Arc::new(ProgressReader::new(redis_client.clone()).await?);

        Ok(Self {
            shared,

            hosts_service,
            backups_service,
            files_service,
            queue_service,
            metrics_service,
            server_service,
            certificate_service,
            producers,
            progress_reader,
            redis_client,
        })
    }
}

//! Job Worker Binary: lance les 3 workers (backup, interactive, maintenance)

use apalis::{
    layers::{retry::RetryPolicy, tracing::TraceLayer},
    prelude::*,
};
use color_eyre::eyre::Result;
use std::sync::Arc;
use tracing::{error, info};
use woodstock::config::Configuration;
use woodstock_server_rs::{
    jobs::{
        config::JobWorkerConfig, layers::job_log::JobLogLayer, state::ApiWorkerState, types::*,
        workers::*,
    },
    logger::init_job_worker_logging,
};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    // Initialize crypto provider for rustls before any TLS operations
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| eyre::eyre!("Failed to install crypto provider"))?;

    let woodstock_config = Arc::new(Configuration::default());

    let state = Arc::new(ApiWorkerState::new(woodstock_config.clone()).await?);

    let job_log_layer = JobLogLayer::new(
        woodstock_config.path.hosts_path.clone(),
        woodstock_config.path.jobs_path.clone(),
    )?;
    init_job_worker_logging(
        &woodstock_config.path.logs_path,
        "job_worker.log",
        job_log_layer,
    )?;
    info!("Starting Job Workers (backup, interactive, maintenance)");

    let worker_cfg = JobWorkerConfig::from_env();

    // Concurrences et retries via JobWorkerConfig
    let backup_concurrency = worker_cfg.backup_concurrency;

    let executors = JobExecutors::new().with_progress(state.progress_publisher.clone());

    // Lancer les 3 workers
    let mut monitor = Monitor::new();
    // backup
    monitor = monitor.register({
        let storage = state.apalis_redis_storage.backup_storage.clone();
        let execs = executors.clone();
        WorkerBuilder::new("backup-worker")
            .data(state.clone())
            .retry(RetryPolicy::retries(3))
            .catch_panic()
            .concurrency(backup_concurrency)
            .layer(
                woodstock_server_rs::jobs::layers::progress::ProgressLayer::new(
                    state.progress_publisher.clone(),
                ),
            )
            .layer(TraceLayer::new())
            .backend(storage)
            .build_fn(
                move |job: BackupQueueJob,
                      task_id: TaskId,
                      state: Data<Arc<ApiWorkerState>>,
                      attempt: Attempt| {
                    let e = execs.clone();
                    async move {
                        let res = match job {
                            BackupQueueJob::Save(data) => {
                                e.handle_backup(task_id, state, data, attempt).await
                            }
                            BackupQueueJob::Remove(data) => {
                                e.handle_remove(task_id, state, data, attempt).await
                            }
                        };
                        res.map_err(|err| {
                            apalis::prelude::Error::from(
                                Box::<dyn std::error::Error + Send + Sync>::from(err),
                            )
                        })
                    }
                },
            )
    });
    // interactive (restore)
    monitor = monitor.register({
        let storage = state.apalis_redis_storage.interactive_storage.clone();
        let execs = executors.clone();
        WorkerBuilder::new("interactive-worker")
            .data(state.clone())
            .retry(RetryPolicy::retries(3))
            .catch_panic()
            .concurrency(worker_cfg.restore_concurrency)
            .layer(
                woodstock_server_rs::jobs::layers::progress::ProgressLayer::new(
                    state.progress_publisher.clone(),
                ),
            )
            .layer(TraceLayer::new())
            .backend(storage)
            .build_fn(
                move |job: RestoreJobData,
                      task_id: TaskId,
                      state: Data<Arc<ApiWorkerState>>,
                      attempt: Attempt| {
                    let e = execs.clone();
                    async move {
                        e.handle_restore(task_id, state, job, attempt)
                            .await
                            .map_err(|err| {
                                apalis::prelude::Error::from(Box::<
                                    dyn std::error::Error + Send + Sync,
                                >::from(
                                    err
                                ))
                            })
                    }
                },
            )
    });
    // maintenance
    monitor = monitor.register({
        let storage = state.apalis_redis_storage.maintenance_storage.clone();
        let execs = executors.clone();
        WorkerBuilder::new("maintenance-worker")
            .data(state.clone())
            .retry(RetryPolicy::retries(3))
            .catch_panic()
            .concurrency(worker_cfg.maintenance_concurrency)
            .layer(
                woodstock_server_rs::jobs::layers::progress::ProgressLayer::new(
                    state.progress_publisher.clone(),
                ),
            )
            .layer(TraceLayer::new())
            .backend(storage)
            .build_fn(
                move |job: MaintenanceJobData,
                      task_id: TaskId,
                      state: Data<Arc<ApiWorkerState>>,
                      attempt: Attempt| {
                    let e = execs.clone();
                    async move {
                        let res = match job {
                            MaintenanceJobData::CleanupPool(data) => {
                                e.handle_cleanup_pool(task_id, state, data, attempt).await
                            }
                            MaintenanceJobData::Fsck(data) => {
                                e.handle_fsck(task_id, state, data, attempt).await
                            }
                            MaintenanceJobData::Stats(_) => e.handle_stats(state, attempt).await,
                        };
                        res.map_err(|err| {
                            apalis::prelude::Error::from(
                                Box::<dyn std::error::Error + Send + Sync>::from(err),
                            )
                        })
                    }
                },
            )
    });

    monitor
        // .shutdown_timeout(Duration::from_secs(5))
        .on_event(|e| match e.inner() {
            apalis::prelude::Event::Error(err) => {
                error!(worker = %e.id().name(), error = %err, "Apalis worker event")
            }
            _ => info!("Event: {e}"),
        })
        .run()
        // .run_with_signal(async {
        //     info!("Monitor started");
        //     tokio::signal::ctrl_c().await?;
        //     info!("Monitor starting shutdown");
        //     Ok(())
        // })
        .await?;

    info!("Job worker shutting down");
    Ok(())
}

//! Scheduler Binary: persiste deux crons (scanner */10, nightly 02:30) vers Redis

use apalis::prelude::*;
use apalis_cron::Schedule;
use color_eyre::eyre::Result;
use eyre::eyre;
use std::{str::FromStr, sync::Arc};
use tokio::sync::Mutex;
use tracing::info;
use woodstock::config::{Configuration, Scheduler};
use woodstock_server_rs::{
    jobs::{producers::*, state::ApiWorkerState, types::*},
    logger::init_logging,
};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    // Initialize crypto provider for rustls before any TLS operations
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| eyre!("Failed to install crypto provider"))?;

    let woodstock_config = Arc::new(Configuration::default());
    let scheduler_service = Scheduler::new(woodstock_config.clone());
    let scheduler = scheduler_service.get_schedule().await?;

    let state = Arc::new(ApiWorkerState::new(woodstock_config.clone()).await?);

    init_logging(&woodstock_config.path.logs_path, "scheduler.log")?;
    info!("Starting Scheduler (persisted cron to Redis)");

    let redis_url = woodstock_config.redis_url();
    info!("Connecting to Redis at: {}", redis_url);

    // Build persisted cron backends
    let scanner_sched = Schedule::from_str(&scheduler.wakeup_schedule)
        .map_err(|e| eyre!("Invalid cron expression for wakeup schedule: {e}"))?;
    let nightly_sched = Schedule::from_str(&scheduler.nightly_schedule)
        .map_err(|e| eyre!("Invalid cron expression for nightly schedule: {e}"))?;

    let scanner_backend = pipe_cron_to_backup_storage(
        scanner_sched,
        state.apalis_redis_storage.schedule_storage.clone(),
    );

    // Nightly cron: utilise un storage dédié (QueueName::Nightly) pour être enregistré
    // comme backend indépendant dans le Monitor. C'est la seule façon de faire tourner
    // le pipe_heartbeat interne à CronPipe (cf. apalis-cron CronPipe::poll).
    let nightly_backend = pipe_cron_to_nightly_storage(
        nightly_sched,
        state.apalis_redis_storage.nightly_storage.clone(),
    );

    // Producers to enqueue jobs when cron ticks are persisted
    let redis_client = redis::Client::open(redis_url.clone())?;
    let producers = Arc::new(Mutex::new(Producers::new(
        state.hosts.clone(),
        state.backups.clone(),
        state.apalis_redis_storage.schedule_storage.clone(),
        state.apalis_redis_storage.backup_storage.clone(),
        state.apalis_redis_storage.interactive_storage.clone(),
        state.apalis_redis_storage.maintenance_storage.clone(),
        state.progress_publisher.clone(),
        redis_client,
    )));

    // Worker consuming the scanner cron stream and producing backup jobs
    let mut monitor = Monitor::new();

    // Scanner: iterate hosts and enqueue backup unique per host
    monitor = monitor.register({
        let producers_scanner = producers.clone();
        WorkerBuilder::new("cron-scanner")
            .data(state.clone())
            .catch_panic()
            .concurrency(1)
            .backend(scanner_backend)
            .build_fn(
                move |_: ScheduleQueueJob, state: Data<Arc<ApiWorkerState>>| {
                    let producers = producers_scanner.clone();
                    async move {
                        let res: eyre::Result<()> = async {
                            let hosts = state.hosts.list_hosts().await?;
                            for host in hosts {
                                // Check if a job is already running for this host
                                let is_running = state.job_utility.is_job_running(&host).await?;
                                if is_running {
                                    info!("Skipping host {}: job already running", host);
                                    continue;
                                }

                                // Check if host is available and should be backuped
                                let should_backup =
                                    state.job_utility.should_backup_host(&host, false).await?;
                                let can_launch = state.job_utility.can_launch_backup(&host).await?;
                                if should_backup && can_launch {
                                    let host_available =
                                        state.job_utility.host_available(&host).await?;

                                    if host_available {
                                        let mut prod = producers.lock().await;
                                        let _ = prod.enqueue_backup_unique(&host, false).await;
                                    }
                                }
                            }
                            Ok(())
                        }
                        .await;
                        res
                    }
                },
            )
    });

    // Nightly: enqueue le job de cleanup refcnt dans maintenance_storage.
    // Le CronPipe doit impérativement être enregistré dans le Monitor pour que
    // pipe_heartbeat (qui pousse les ticks dans Redis) soit pollé.
    monitor = monitor.register({
        let producers_nightly = producers.clone();
        WorkerBuilder::new("cron-nightly")
            .catch_panic()
            .concurrency(1)
            .backend(nightly_backend)
            .build_fn(move |_: ScheduleQueueJob| {
                let producers = producers_nightly.clone();
                async move {
                    let mut prod = producers.lock().await;
                    let res = prod.enqueue_cleanup_refcnt().await;
                    if let Err(e) = &res {
                        tracing::error!("Failed to enqueue nightly cleanup refcnt: {e}");
                    } else {
                        info!("Nightly cleanup refcnt job enqueued");
                    }
                    res.map(|_| ())
                }
            })
    });

    monitor.run().await?;

    info!("Scheduler shutting down");
    Ok(())
}

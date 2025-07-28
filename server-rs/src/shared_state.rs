//! Infrastructure state shared between the API server and the job worker.
//!
//! Both binaries need the same core resources (Redis, configuration, resolver,
//! job utilities). [`SharedState`] groups them so the initialisation code is
//! written only once.

use eyre::{Result, WrapErr};
use redis::aio::ConnectionManager;
use std::sync::Arc;
use tracing::info;
use woodstock::{
    config::{Backups, Configuration, Hosts, Scheduler},
    server::{job::JobUtility, resolve::SocketAddrResolver},
};

/// Infrastructure state shared between the API server and the job worker.
///
/// Create with [`SharedState::new`] and then embed it inside
/// [`crate::api::ApiServerState`] or [`crate::jobs::ApiWorkerState`].
/// Both types implement [`std::ops::Deref<Target = SharedState>`] so you can
/// access shared fields directly (e.g. `state.config`, `state.hosts`).
#[derive(Clone)]
pub struct SharedState {
    /// Global application configuration.
    pub config: Arc<Configuration>,
    /// Cron / periodic-job scheduler configuration.
    pub scheduler: Arc<Scheduler>,
    /// Host configuration registry (reads `hosts.yml`).
    pub hosts: Arc<Hosts>,
    /// Backup metadata registry (reads per-host backup directories).
    pub backups: Arc<Backups>,
    /// DNS / IP resolver backed by Redis.
    pub resolver: Arc<SocketAddrResolver>,
    /// High-level job orchestration utilities (availability checks, ping, …).
    pub job_utility: Arc<JobUtility>,
}

impl SharedState {
    /// Initialise shared infrastructure from the given configuration.
    ///
    /// Also returns the underlying [`redis::Client`] so the caller can open
    /// additional connections (job producers, progress publisher, …) without
    /// creating a second connection pool.
    ///
    /// # Errors
    ///
    /// Returns an error if Redis is unreachable or if any sub-component fails
    /// to initialise.
    pub async fn new(config: Arc<Configuration>) -> Result<(Self, redis::Client)> {
        let redis_url = config.redis_url();
        info!("Connecting to Redis: {}", redis_url);
        let redis_client =
            redis::Client::open(redis_url).wrap_err("Failed to open Redis connection")?;

        let scheduler = Arc::new(Scheduler::new(config.clone()));
        // Dedicated ConnectionManager for the hosts cache (list + per-host config).
        let hosts_cache_conn = ConnectionManager::new(redis_client.clone())
            .await
            .wrap_err("Failed to create ConnectionManager for hosts cache")?;
        let hosts = Arc::new(Hosts::with_redis_conn(
            config.clone(),
            scheduler.clone(),
            hosts_cache_conn,
        ));
        // Dedicated ConnectionManager used exclusively for publishing BackupChangedEvent
        // notifications to Redis. We intentionally do not reuse other shared Redis
        // connections here to:
        //   - isolate backup notification traffic from other Redis workloads
        //     (e.g. resolver lookups, job utilities), and
        //   - avoid a slow or backpressured publisher interfering with unrelated
        //     commands on shared connections.
        // The underlying redis::Client is still shared; only this managed connection
        // is dedicated to the backup notification publisher.
        let backup_conn = ConnectionManager::new(redis_client.clone())
            .await
            .wrap_err("Failed to create ConnectionManager for backup notifier")?;
        let backups = Arc::new(Backups::with_redis_publisher(config.clone(), backup_conn));
        let resolver = Arc::new(
            SocketAddrResolver::new(redis_client.clone())
                .wrap_err("Failed to create SocketAddrResolver")?,
        );
        let job_utility = Arc::new(
            JobUtility::new(
                config.clone(),
                hosts.clone(),
                backups.clone(),
                scheduler.clone(),
                resolver.clone(),
            )
            .wrap_err("Failed to create JobUtility")?,
        );

        Ok((
            Self {
                config,
                scheduler,
                hosts,
                backups,
                resolver,
                job_utility,
            },
            redis_client,
        ))
    }
}

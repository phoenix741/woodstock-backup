use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use eyre::{eyre, Result};
use redis::aio::ConnectionManager;
use tokio::{fs::read_to_string, sync::Mutex};
use tracing::{debug, warn};

use super::{Configuration, HostConfiguration, Schedule, Scheduler};
use crate::utils::cache::{cache_invalidate, cache_wrap};

/// # Hosts Configuration Module
///
/// This module provides the [`Hosts`] struct and associated methods for managing host configuration
/// files in the Woodstock backup system. It is responsible for listing available hosts, loading
/// host-specific configuration, and reading host configuration files from disk.
///
/// ## Main Structure
///
/// - [`Hosts`]: Central struct for managing host configuration files.
///
/// ## Key Methods
///
/// - [`Hosts::new`]: Create a new `Hosts` manager from a configuration.
/// - [`Hosts::list_hosts`]: List all hostnames known to the system.
/// - [`Hosts::get_host`]: Load the configuration for a specific host.
/// - [`Hosts::read_host_file`]: Read and parse a host configuration file.
///
/// ## Error Handling
///
/// - Most methods return `Result` and propagate I/O or deserialization errors using the `eyre` crate.
/// - If a host is not found, an error is returned.
///
/// ## Panics
///
/// - Methods do not panic under normal operation. Errors are returned as `Result`.
///
/// ## Thread Safety
///
/// This struct is not thread-safe by itself. If used in a concurrent context, wrap in a mutex or use only from one thread.
///
/// ## See Also
///
/// - [`HostConfiguration`]: For host-specific settings

/// Redis cache key for the flat host list (`hosts.yml`).
const HOSTS_LIST_KEY: &str = "woodstock:cache:hosts";

/// TTL (in seconds) for host cache entries — 24 hours.
const CACHE_TTL_SECS: u64 = 86400;

/// Returns the Redis cache key for the configuration of a single host.
fn host_config_key(hostname: &str) -> String {
    format!("woodstock:cache:host:{hostname}")
}

pub struct Hosts {
    config: Arc<Configuration>,
    /// Scheduler
    scheduler: Arc<Scheduler>,
    /// Optional Redis connection used to cache host list and per-host config.
    /// `None` when running from the CLI (no Redis available).
    redis_conn: Option<Arc<Mutex<ConnectionManager>>>,
}

impl Hosts {
    /// Creates a new `Hosts` manager from the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Reference to the Woodstock [`Configuration`] struct.
    ///
    /// # Returns
    ///
    /// A new instance of [`Hosts`] with paths initialized from the configuration.
    #[must_use]
    pub fn new(config: Arc<Configuration>, scheduler: Arc<Scheduler>) -> Self {
        Self {
            config,
            scheduler,
            redis_conn: None,
        }
    }

    /// Creates a `Hosts` instance that caches `hosts.yml` and per-host
    /// configuration in Redis, reducing disk I/O between backup operations.
    ///
    /// Use this variant from server binaries that have access to a Redis
    /// [`ConnectionManager`]. The CLI uses [`Self::new`] (no Redis).
    #[must_use]
    pub fn with_redis_conn(
        config: Arc<Configuration>,
        scheduler: Arc<Scheduler>,
        conn: ConnectionManager,
    ) -> Self {
        Self {
            config,
            scheduler,
            redis_conn: Some(Arc::new(Mutex::new(conn))),
        }
    }

    /// Lists all hostnames known to the system.
    ///
    /// Reads the `hosts.yml` file and returns a vector of hostnames.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - List of hostnames if the file is readable and valid.
    /// * `Err(eyre::Report)` - If the file cannot be read or parsed (returns empty list on error).
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be parsed as YAML, but not if the file is missing.
    pub async fn list_hosts(&self) -> Result<Vec<String>> {
        if let Some(conn) = &self.redis_conn {
            let path = self.config.path.config_path_hosts.clone();
            let result = cache_wrap(conn, HOSTS_LIST_KEY, CACHE_TTL_SECS, || async move {
                Self::read_hosts_from_disk(&path).await
            })
            .await;
            return Ok(result);
        }
        Ok(Self::read_hosts_from_disk(&self.config.path.config_path_hosts).await)
    }

    /// Reads `hosts.yml` from disk and returns the list of hostnames.
    async fn read_hosts_from_disk(path: &Path) -> Vec<String> {
        debug!("Reading hosts from {:?}", path);
        let hosts = read_to_string(path).await;
        match hosts {
            Ok(content) => {
                debug!("Hosts file content: {content}");
                serde_yaml_ng::from_str(&content).unwrap_or_else(|e| {
                    warn!("Failed to parse hosts file: {e}");
                    vec![]
                })
            }
            Err(e) => {
                warn!("Error reading hosts file: {e}");
                vec![]
            }
        }
    }

    /// Loads the configuration for a specific host.
    ///
    /// Checks if the host exists in the list, then loads its configuration file.
    ///
    /// # Arguments
    ///
    /// * `hostname` - The name of the host to load.
    ///
    /// # Returns
    ///
    /// * `Ok(HostConfiguration)` - The configuration for the host if found and valid.
    /// * `Err(eyre::Report)` - If the host is not found or the file cannot be read/parsed.
    ///
    /// # Errors
    ///
    /// Returns an error if the host is not listed or the configuration file is invalid.
    pub async fn get_host(&self, hostname: &str) -> Result<HostConfiguration> {
        // Check if the host is in the list (uses cache if Redis is available)
        let hosts = self.list_hosts().await?;
        if !hosts.contains(&hostname.to_string()) {
            return Err(eyre!("Host {hostname} not found"));
        }

        if let Some(conn) = &self.redis_conn {
            let key = host_config_key(hostname);
            let config_path = self.get_host_configuration_file(hostname);
            let result: Option<HostConfiguration> =
                cache_wrap(conn, &key, CACHE_TTL_SECS, || async move {
                    read_to_string(&config_path)
                        .await
                        .ok()
                        .and_then(|c| serde_yaml_ng::from_str(&c).ok())
                })
                .await;
            return result.ok_or_else(|| eyre!("Failed to load config for host {hostname}"));
        }

        let path = self.get_host_configuration_file(hostname);
        self.read_host_file(path).await
    }

    /// Reads and parses a host configuration file from disk.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the host configuration file (can be any type implementing `AsRef<Path>`).
    ///
    /// # Returns
    ///
    /// * `Ok(HostConfiguration)` - The parsed host configuration.
    /// * `Err(eyre::Report)` - If the file cannot be read or parsed as YAML.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub async fn read_host_file<P: AsRef<Path>>(&self, path: P) -> Result<HostConfiguration> {
        let content = read_to_string(path).await?;
        let host: HostConfiguration = serde_yaml_ng::from_str(&content)?;

        Ok(host)
    }

    /// Returns the path to the configuration file for a given host.
    fn get_host_configuration_file(&self, hostname: &str) -> PathBuf {
        self.config.path.config_path.join(format!("{hostname}.yml"))
    }

    /// Invalidates the cached host list (`woodstock:cache:hosts`).
    ///
    /// The next call to [`Self::list_hosts`] will re-read `hosts.yml` from disk.
    pub async fn invalidate_hosts_list_cache(&self) {
        if let Some(conn) = &self.redis_conn {
            cache_invalidate(conn, HOSTS_LIST_KEY).await;
        }
    }

    /// Invalidates the cached configuration for a single host
    /// (`woodstock:cache:host:{hostname}`).
    ///
    /// The next call to [`Self::get_host`] for this hostname will re-read
    /// `{hostname}.yml` from disk.
    pub async fn invalidate_host_config_cache(&self, hostname: &str) {
        if let Some(conn) = &self.redis_conn {
            cache_invalidate(conn, &host_config_key(hostname)).await;
        }
    }

    /// Fusionne le schedule global avec celui du host (host override global).
    pub async fn get_schedule(&self, hostname: &str) -> Result<Schedule> {
        let global_scheduler = self.scheduler.get_schedule().await?;
        let global_scheduler = global_scheduler.default_schedule;

        let config = self.get_host(hostname).await?;
        let mut scheduler = config.schedule.unwrap_or_else(|| global_scheduler.clone());

        scheduler.activated = scheduler.activated.or(global_scheduler.activated);
        scheduler.backup_period = scheduler.backup_period.or(global_scheduler.backup_period);
        scheduler.backup_to_keep = scheduler.backup_to_keep.or(global_scheduler.backup_to_keep);

        debug!("Schedule for host {hostname}: {scheduler:?}");

        Ok(scheduler)
    }
}

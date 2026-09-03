use std::{net::SocketAddr, sync::Arc};

use chrono::{DateTime, Duration, Local};
use eyre::Result;
use futures::{stream, StreamExt};
use tracing::debug;

use crate::{
    config::{
        blackout_status_at, Backups, Configuration, HostConfiguration, Hosts,
        DNS_RESOLVE_MAX_CONCURRENCY,
    },
    server::{
        resolve::{resolve_dns_async, SocketAddrResolver},
        tools::ping,
    },
    utils::lock_redis::{LockOperation, PoolLockOperation, PoolLockRedis},
};

/// Central structure for job utility
pub struct JobUtility {
    configuration: Arc<Configuration>,
    hosts_config: Arc<Hosts>,
    backups_config: Arc<Backups>,
    resolver: Arc<SocketAddrResolver>,
}

impl JobUtility {
    /// Creates a new instance of `JobUtility`.
    #[must_use]
    pub fn new(
        configuration: Arc<Configuration>,
        hosts_config: Arc<Hosts>,
        backups_config: Arc<Backups>,
        resolver: Arc<SocketAddrResolver>,
    ) -> Result<Self> {
        Ok(Self {
            hosts_config,
            backups_config,
            resolver,
            configuration,
        })
    }

    #[must_use]
    pub async fn get_time_to_next_backup(&self, hostname: &str) -> Result<Option<Duration>> {
        let schedule = self.hosts_config.get_schedule(hostname).await?;
        if !schedule.activated.unwrap_or_default() {
            debug!("No active schedule for host: {}", hostname);
            return Ok(None);
        }

        let Some(time_since_last_backup) = self
            .backups_config
            .get_time_since_last_backup(hostname)
            .await
        else {
            debug!("No last backup found for host: {}", hostname);
            return Ok(Some(Duration::zero()));
        };
        let Some(last_backup) = self.backups_config.get_last_backup(hostname).await else {
            debug!("No last backup found for host: {}", hostname);
            return Ok(None);
        };
        if last_backup.status.is_aborted() {
            debug!(
                "Last backup for host {} was {:?}, treating as no backup",
                hostname, last_backup.status
            );
            return Ok(Some(Duration::zero()));
        }

        let backup_period = schedule.backup_period.unwrap_or_default();
        debug!("Last backup for host {} have ben made at {} hours past (should be made after {} hours)", hostname, time_since_last_backup.num_hours(), backup_period / 3600);

        let time_to_next_backup = Duration::seconds(backup_period) - time_since_last_backup;
        Ok(Some(time_to_next_backup.max(Duration::zero())))
    }

    #[must_use]
    pub async fn get_date_to_next_backup(&self, hostname: &str) -> Result<Option<DateTime<Local>>> {
        let Some(duration) = self.get_time_to_next_backup(hostname).await? else {
            return Ok(None);
        };

        let next_date = Local::now() + duration;

        // If the natural next-attempt date falls inside a blackout window, the actual
        // attempt won't happen before the window ends — reflect that in the displayed date
        // (informational only: the override logic in `is_in_blackout_now` is for the
        // scheduler's decision at attempt time, not for this display value).
        let schedule = self.hosts_config.get_schedule(hostname).await?;
        let next_date = match schedule.blackout.as_ref().filter(|w| !w.is_empty()) {
            Some(windows) => blackout_status_at(next_date, windows).unwrap_or(next_date),
            None => next_date,
        };

        Ok(Some(next_date))
    }

    /// Returns `Some(retry_at)` if `host` should not be scheduled right now because of an
    /// active blackout window, or `None` if scheduling may proceed (no active window, or the
    /// host is late enough for `blackout_override_after_periods` to override it).
    ///
    /// `retry_at` is the earliest moment scheduling could legitimately be retried: either the
    /// window's natural end, or the moment `blackout_override_after_periods` will kick in,
    /// whichever comes first — callers (the scanner's dynamic-wakeup computation) rely on this
    /// being the *exact* next useful moment, not an arbitrary short backoff, otherwise a
    /// multi-hour blackout window turns into a busy-poll for its whole duration.
    ///
    /// Distinct from [`Self::can_launch_backup`] on purpose: this is a calendar gate on
    /// *starting a new* backup, not a lock-based gate on pool operations.
    pub async fn is_in_blackout_now(&self, host: &str) -> Result<Option<DateTime<Local>>> {
        let schedule = self.hosts_config.get_schedule(host).await?;
        let Some(windows) = schedule.blackout.as_ref().filter(|w| !w.is_empty()) else {
            return Ok(None);
        };

        let now = Local::now();
        let Some(blackout_end) = blackout_status_at(now, windows) else {
            return Ok(None);
        };

        let Some(ratio) = schedule.blackout_override_after_periods else {
            debug!("Host {host} is in blackout until {blackout_end} (no override configured)");
            return Ok(Some(blackout_end));
        };

        let backup_period = schedule.backup_period.unwrap_or_default();
        if backup_period <= 0 {
            return Ok(Some(blackout_end));
        }

        let Some(last_backup) = self.backups_config.get_last_backup(host).await else {
            debug!("Host {host}: never backed up, overriding blackout for the first backup");
            return Ok(None);
        };
        if last_backup.status.is_aborted() {
            debug!(
                "Host {host}: last backup was aborted, overriding blackout to retry immediately"
            );
            return Ok(None);
        }
        let Some(end_date) = last_backup.end_date else {
            return Ok(Some(blackout_end));
        };

        let override_at =
            end_date + Duration::seconds((backup_period as f64 * f64::from(ratio)) as i64);
        if now >= override_at {
            debug!("Host {host} passed its blackout override threshold at {override_at}, overriding blackout");
            return Ok(None);
        }

        let retry_at = blackout_end.min(override_at);
        debug!("Host {host} is in blackout until {retry_at} (window end {blackout_end}, override at {override_at})");
        Ok(Some(retry_at))
    }

    pub async fn resolve_from_config(
        &self,
        hostname: &str,
        configuration: &HostConfiguration,
    ) -> Result<Vec<SocketAddr>> {
        let port = configuration.port;
        if let Some(ref addresses) = configuration.addresses {
            debug!("Resolving addresses from configuration for {hostname}");

            // Resolve each entry asynchronously without blocking the runtime.
            // Limit concurrency to avoid saturating the blocking pool.
            let resolved_addresses: Vec<SocketAddr> = stream::iter(addresses.iter().cloned())
                .map(|entry| async move { resolve_dns_async(&entry).await })
                .buffer_unordered(DNS_RESOLVE_MAX_CONCURRENCY)
                .flat_map(stream::iter)
                .map(|ip| SocketAddr::new(ip, port))
                .collect()
                .await;

            Ok(resolved_addresses)
        } else {
            let addr = self.resolver.resolve(hostname, port).await?;
            Ok(addr)
        }
    }

    pub async fn ping_from_config(
        &self,
        hostname: &str,
        configuration: &HostConfiguration,
    ) -> Option<SocketAddr> {
        let Ok(addresses) = self.resolve_from_config(hostname, configuration).await else {
            debug!("Failed to resolve addresses for {hostname}");
            return None;
        };

        for addr in addresses {
            if ping(&addr, hostname, self.configuration.clone()).await {
                return Some(addr);
            }
        }

        None
    }

    pub async fn should_backup_host(&self, host: &str, force: bool) -> Result<bool> {
        if force {
            return Ok(true);
        }

        let time_to_next_backup = self.get_time_to_next_backup(host).await?;
        debug!("Time to next backup for host {host}: {time_to_next_backup:?}");

        Ok(time_to_next_backup.is_some_and(|t| t.is_zero()))
    }

    /// Vérifie si le scheduler peut lancer une nouvelle sauvegarde maintenant.
    ///
    /// Cette décision est volontairement distincte de `should_backup_host`: une sauvegarde
    /// déjà lancée ne doit pas être bloquée par ce test. Ici, on évite seulement de planifier
    /// un nouveau backup pendant une opération longue sur le pool, comme un fsck.
    pub async fn can_launch_backup(&self, host: &str) -> Result<bool> {
        let redis_url = self.configuration.redis_url();
        if let Some(lock) = PoolLockRedis::active_exclusive_lock_with_path(
            &redis_url,
            &self.configuration.path.pool_path,
        )
        .await?
        {
            if lock.operation_name == Some(LockOperation::Pool(PoolLockOperation::Fsck)) {
                debug!(
                    "Skip scheduling backup for host {}: pool fsck lock is active",
                    host
                );
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Vérifie la disponibilité réseau & retourne l'adresse ip:port atteignable si trouvée.
    pub async fn host_available(&self, host: &str) -> Result<bool> {
        let host_conf = self.hosts_config.get_host(host).await?;

        let is_host_available = self.ping_from_config(host, &host_conf).await;

        if is_host_available.is_none() {
            debug!("Host {host} is not reachable");
        }

        Ok(is_host_available.is_some())
    }

    /// Vérifie si un job (backup, remove) est déjà en cours d'exécution pour l'hôte donné.
    /// Utilise le système de lock Redis pour déterminer si un job est actif.
    pub async fn is_job_running(&self, host: &str) -> Result<bool> {
        let redis_url = self.configuration.redis_url();

        // Passive inspection only: do not acquire a temporary lock just to probe state,
        // otherwise the probe itself creates a short-lived exclusive lock and releases it
        // asynchronously via Drop.
        PoolLockRedis::has_active_lock(&redis_url, host).await
    }
}

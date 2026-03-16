use std::{net::SocketAddr, str::FromStr, sync::Arc};

use chrono::{DateTime, Duration, Local};
use eyre::Result;
use futures::{stream, StreamExt};
use tracing::debug;

use crate::{
    config::{
        Backups, Configuration, HostConfiguration, Hosts, Scheduler, DNS_RESOLVE_MAX_CONCURRENCY,
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
    scheduler: Arc<Scheduler>,
    resolver: Arc<SocketAddrResolver>,
}

impl JobUtility {
    /// Creates a new instance of `JobUtility`.
    #[must_use]
    pub fn new(
        configuration: Arc<Configuration>,
        hosts_config: Arc<Hosts>,
        backups_config: Arc<Backups>,
        scheduler: Arc<Scheduler>,
        resolver: Arc<SocketAddrResolver>,
    ) -> Result<Self> {
        Ok(Self {
            hosts_config,
            backups_config,
            scheduler,
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

        let scheduler = self.scheduler.get_schedule().await?;
        let scheduler = scheduler.wakeup_schedule;

        let next_date = Local::now() + duration;

        // Use cron to calculate the next wakeup time
        debug!("Calculate schedule with the scheduler: {scheduler}");
        let wakeup_scheduler = cron::Schedule::from_str(&scheduler)?;
        let next_date = wakeup_scheduler.after(&next_date).next();

        Ok(next_date)
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

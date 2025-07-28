use async_graphql::ID;
use async_graphql::{Context, Object, Result as GqlResult};
use chrono::Local;
use std::str::FromStr;
use tracing::debug;
use uuid::Uuid;

use crate::api::dto::{
    ApplicationEvent, BigIntTimeSerie, DiskUsage as GqlDiskUsage, GqlStatistics, Host,
    HostStatistics as GqlHostStatistics, Job, NumberTimeSerie, PoolHealthStatusDto,
    PoolUsage as GqlPoolUsage, QueueListInput, QueueStats,
    ServerInformations as GqlServerInformations,
};
use crate::api::ApiServerState;
use crate::graphql::resolvers::types::BackupEx;
use crate::graphql::scalars::BigIntScalar;

use woodstock::events::read_events;
use woodstock::pool::PoolManager;

#[derive(Default)]
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hosts(&self, ctx: &Context<'_>) -> GqlResult<Vec<Host>> {
        let state = ctx.data::<ApiServerState>()?;
        let hosts = state
            .hosts_service
            .list_hosts()
            .await
            .map_err(super::util::map_err)?;

        Ok(hosts
            .into_iter()
            .map(|name| Host { name: ID(name) })
            .collect())
    }

    async fn host(&self, ctx: &Context<'_>, hostname: String) -> GqlResult<Host> {
        let state = ctx.data::<ApiServerState>()?;
        let hosts = state
            .hosts_service
            .list_hosts()
            .await
            .map_err(super::util::map_err)?;

        if !hosts.contains(&hostname) {
            return Err(async_graphql::Error::new(format!(
                "Can't find the host with the name {hostname}"
            )));
        }

        Ok(Host { name: ID(hostname) })
    }

    async fn backups(&self, ctx: &Context<'_>, hostname: String) -> GqlResult<Vec<BackupEx>> {
        let state = ctx.data::<ApiServerState>()?;
        let hosts = state
            .hosts_service
            .list_hosts()
            .await
            .map_err(super::util::map_err)?;

        if !hosts.contains(&hostname) {
            return Err(async_graphql::Error::new(format!(
                "Can't find the host with the name {hostname}"
            )));
        }

        let backups = state
            .backups_service
            .get_backups(&hostname)
            .await
            .map_err(super::util::map_err)?;

        Ok(backups
            .into_iter()
            .map(|b| BackupEx {
                hostname: hostname.clone(),
                inner: b,
            })
            .collect())
    }

    /// Récupère tous les backups ayant échoué (avec error_message non null)
    async fn failed_backups(&self, ctx: &Context<'_>) -> GqlResult<Vec<BackupEx>> {
        let state = ctx.data::<ApiServerState>()?;
        let hosts = state
            .hosts_service
            .list_hosts()
            .await
            .map_err(super::util::map_err)?;

        let mut failed_backups = Vec::new();

        for hostname in hosts {
            let backups = state
                .backups_service
                .get_backups(&hostname)
                .await
                .map_err(super::util::map_err)?;

            for backup in backups {
                if backup.error_message.is_some() {
                    failed_backups.push(BackupEx {
                        hostname: hostname.clone(),
                        inner: backup,
                    });
                }
            }
        }

        Ok(failed_backups)
    }

    async fn backup(&self, ctx: &Context<'_>, hostname: String, id: String) -> GqlResult<BackupEx> {
        let backup_id = Uuid::parse_str(&id)
            .map_err(|_| async_graphql::Error::new(format!("Invalid backup UUID: {id}")))?;
        let state = ctx.data::<ApiServerState>()?;
        let hosts = state
            .hosts_service
            .list_hosts()
            .await
            .map_err(super::util::map_err)?;

        if !hosts.contains(&hostname) {
            return Err(async_graphql::Error::new(format!(
                "Can't find the host with the name {hostname}"
            )));
        }

        let backup = state
            .backups_service
            .get_backup(&hostname, backup_id)
            .await
            .map_err(super::util::map_err)?
            .ok_or_else(|| {
                async_graphql::Error::new(format!(
                    "Can't find the backup {id} for the host {hostname}"
                ))
            })?;

        Ok(BackupEx {
            hostname,
            inner: backup,
        })
    }

    async fn queue_stats(&self, ctx: &Context<'_>) -> GqlResult<QueueStats> {
        let state = ctx.data::<ApiServerState>()?;

        // Compute counts from the ProgressReader (same source as the `queue` list query)
        // so the badges are always coherent with the displayed task cards.
        let all_jobs = state
            .progress_reader
            .list(crate::jobs::progress::ProgressFilter::default())
            .await
            .map_err(super::util::map_err)?;

        let pending = all_jobs
            .iter()
            .filter(|j| j.status == crate::jobs::progress::JobStatus::Created)
            .count();
        let running = all_jobs
            .iter()
            .filter(|j| j.status == crate::jobs::progress::JobStatus::Started)
            .count();
        let success = all_jobs
            .iter()
            .filter(|j| j.status == crate::jobs::progress::JobStatus::Completed)
            .count();
        let failed = all_jobs
            .iter()
            .filter(|j| j.status == crate::jobs::progress::JobStatus::Failed)
            .count();

        let (last_execution, next_wakeup) = {
            let sched = state
                .scheduler
                .get_schedule()
                .await
                .map_err(super::util::map_err)?;

            match apalis_cron::Schedule::from_str(&sched.wakeup_schedule) {
                Ok(schedule) => {
                    let now = Local::now();
                    let next_dt = schedule.after(&now).next();
                    let prev_dt = schedule.after(&now).next_back();
                    (prev_dt, next_dt)
                }
                Err(_) => (None, None),
            }
        };

        Ok(QueueStats {
            pending,
            running,
            success,
            failed,
            dead: 0, // not tracked in ProgressReader
            last_execution,
            next_wakeup,
        })
    }

    async fn queue(&self, ctx: &Context<'_>, input: QueueListInput) -> GqlResult<Vec<Job>> {
        let state = ctx.data::<ApiServerState>()?;
        debug!("Listing jobs with input: {:?}", input);
        let jobs = state
            .progress_reader
            .list(input.into())
            .await
            .map_err(super::util::map_err)?;
        Ok(jobs.into_iter().map(Into::into).collect())
    }

    async fn events(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "firstEvent")] first_event: chrono::DateTime<Local>,
        #[graphql(name = "lastEvent")] last_event: chrono::DateTime<Local>,
        // Maximum number of events to return (default: 50, max: 500)
        limit: Option<i32>,
        // Pagination offset (default: 0)
        offset: Option<i32>,
    ) -> GqlResult<Vec<ApplicationEvent>> {
        let state = ctx.data::<ApiServerState>()?;
        let start_date = first_event.date_naive();
        let end_date = last_event.date_naive();
        let events = read_events(
            &state.config,
            state.config.path.events_path.clone(),
            start_date,
            end_date,
        )
        .await
        .map_err(super::util::map_err)?;
        let mut list: Vec<ApplicationEvent> = events.into_iter().map(Into::into).collect();
        list.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Server-side pagination: reduces the amount of data sent to the client
        let skip = offset.unwrap_or(0).max(0) as usize;
        let take = limit.unwrap_or(50).clamp(1, 500) as usize;
        let list = list.into_iter().skip(skip).take(take).collect();

        Ok(list)
    }

    async fn informations(&self, _ctx: &Context<'_>) -> GqlResult<GqlServerInformations> {
        let hostname = gethostname::gethostname().to_string_lossy().to_string();
        let uptime = sysinfo::System::uptime();
        let woodstock_version = env!("CARGO_PKG_VERSION").to_string();
        Ok(GqlServerInformations {
            hostname,
            uptime,
            woodstock_version,
        })
    }

    /// Gets the health status of the storage pool.
    /// Checks for dirty state (crashed refcnt operations).
    async fn pool_health(&self, ctx: &Context<'_>) -> GqlResult<PoolHealthStatusDto> {
        let state = ctx.data::<ApiServerState>()?;

        let pool = PoolManager::new(state.config.clone());
        let pending_count = pool.count_pending().await.map_err(super::util::map_err)?;
        let is_dirty = pool
            .is_dirty()
            .await
            .map_err(super::util::map_err)?
            .is_some();

        Ok(PoolHealthStatusDto {
            healthy: !is_dirty,
            is_dirty,
            pending_count: pending_count as i32,
        })
    }

    async fn statistics(&self, _ctx: &Context<'_>) -> GqlResult<GqlStatistics> {
        Ok(GqlStatistics)
    }
}

// Statistics sub-object resolvers
#[Object(name = "Statistics")]
impl GqlStatistics {
    async fn disk_usage(&self, ctx: &Context<'_>) -> GqlResult<GqlDiskUsage> {
        let state = ctx.data::<ApiServerState>()?;
        let usage = woodstock::statistics::instant_stats::get_space(&state.config.path.pool_path)
            .map_err(super::util::map_err)?;
        let mut history =
            woodstock::statistics::disk_stats::read_disk_history(&state.config.path.pool_path)
                .await;
        history.sort_unstable_by_key(|h| h.date);
        let last_month = super::util::find_last_month(&history);

        Ok(GqlDiskUsage {
            used: BigIntScalar(usage.used),
            used_last_month: last_month.map(|h| BigIntScalar(h.used)).unwrap_or_default(),
            used_range: history
                .iter()
                .map(|h| BigIntTimeSerie {
                    time: h.date,
                    value: BigIntScalar(h.used),
                })
                .collect(),
            free: BigIntScalar(usage.free),
            free_last_month: last_month.map(|h| BigIntScalar(h.free)).unwrap_or_default(),
            free_range: history
                .iter()
                .map(|h| BigIntTimeSerie {
                    time: h.date,
                    value: BigIntScalar(h.free),
                })
                .collect(),
            total: BigIntScalar(usage.size),
            total_last_month: last_month.map(|h| BigIntScalar(h.size)).unwrap_or_default(),
            total_range: history
                .iter()
                .map(|h| BigIntTimeSerie {
                    time: h.date,
                    value: BigIntScalar(h.size),
                })
                .collect(),
        })
    }

    async fn pool_usage(&self, ctx: &Context<'_>) -> GqlResult<GqlPoolUsage> {
        let state = ctx.data::<ApiServerState>()?;
        use woodstock::statistics::{load_history, read_statistics};
        let stats = read_statistics(&state.config.path.pool_path).await;
        let mut history = load_history(&state.config.path.pool_path).await;
        history.sort_unstable_by_key(|h| h.date);

        let last_month = super::util::find_last_month(&history);

        Ok(GqlPoolUsage {
            longest_chain: stats.longest_chain as i32,
            longest_chain_range: history
                .iter()
                .map(|h| NumberTimeSerie {
                    time: h.date,
                    value: h.longest_chain as i32,
                })
                .collect(),
            longest_chain_last_month: last_month.map(|m| m.longest_chain as i32),

            nb_chunk: stats.nb_chunk as i32,
            nb_chunk_range: history
                .iter()
                .map(|h| NumberTimeSerie {
                    time: h.date,
                    value: h.nb_chunk as i32,
                })
                .collect(),
            nb_chunk_last_month: last_month.map(|m| m.nb_chunk as i32),

            nb_ref: stats.nb_ref as i32,
            nb_ref_range: history
                .iter()
                .map(|h| NumberTimeSerie {
                    time: h.date,
                    value: h.nb_ref as i32,
                })
                .collect(),
            nb_ref_last_month: last_month.map(|m| m.nb_ref as i32),

            size: BigIntScalar(stats.size),
            size_range: history
                .iter()
                .map(|h| BigIntTimeSerie {
                    time: h.date,
                    value: BigIntScalar(h.size),
                })
                .collect(),
            size_last_month: BigIntScalar(last_month.map(|m| m.size).unwrap_or(0)),

            compressed_size: BigIntScalar(stats.compressed_size),
            compressed_size_range: history
                .iter()
                .map(|h| BigIntTimeSerie {
                    time: h.date,
                    value: BigIntScalar(h.compressed_size),
                })
                .collect(),
            compressed_size_last_month: BigIntScalar(
                last_month.map(|m| m.compressed_size).unwrap_or(0),
            ),

            unused_size: BigIntScalar(stats.unused_size),
            unused_size_range: history
                .iter()
                .map(|h| BigIntTimeSerie {
                    time: h.date,
                    value: BigIntScalar(h.unused_size),
                })
                .collect(),
            unused_size_last_month: BigIntScalar(last_month.map(|m| m.unused_size).unwrap_or(0)),
        })
    }

    async fn hosts(&self, ctx: &Context<'_>) -> GqlResult<Vec<GqlHostStatistics>> {
        let state = ctx.data::<ApiServerState>()?;
        let hosts = state
            .hosts_service
            .list_hosts()
            .await
            .map_err(super::util::map_err)?;
        use std::path::PathBuf;
        use woodstock::statistics::{load_history, read_statistics};
        use woodstock::statistics::{HistoricalPoolStatistics, PoolStatistics};

        let mut res = Vec::with_capacity(hosts.len());
        for host in hosts {
            let host_dir: PathBuf = state.backups_service.get_host_path(&host);
            let stats: PoolStatistics = read_statistics(&host_dir).await;
            let mut history: Vec<HistoricalPoolStatistics> = load_history(&host_dir).await;
            history.sort_unstable_by_key(|h| h.date);
            let last_month = super::util::find_last_month(&history);

            res.push(GqlHostStatistics {
                host: host.clone(),
                longest_chain: stats.longest_chain as i32,
                longest_chain_range: history
                    .iter()
                    .map(|h| NumberTimeSerie {
                        time: h.date,
                        value: h.longest_chain as i32,
                    })
                    .collect(),
                longest_chain_last_month: last_month.map(|e| e.longest_chain as i32),

                nb_chunk: stats.nb_chunk as i32,
                nb_chunk_range: history
                    .iter()
                    .map(|h| NumberTimeSerie {
                        time: h.date,
                        value: h.nb_chunk as i32,
                    })
                    .collect(),
                nb_chunk_last_month: last_month
                    .map(|e: &HistoricalPoolStatistics| e.nb_chunk as i32),

                nb_ref: stats.nb_ref as i32,
                nb_ref_range: history
                    .iter()
                    .map(|h| NumberTimeSerie {
                        time: h.date,
                        value: h.nb_ref as i32,
                    })
                    .collect(),
                nb_ref_last_month: last_month.map(|e| e.nb_ref as i32),

                size: BigIntScalar(stats.size),
                size_range: history
                    .iter()
                    .map(|h| BigIntTimeSerie {
                        time: h.date,
                        value: BigIntScalar(h.size),
                    })
                    .collect(),
                size_last_month: BigIntScalar(last_month.map(|m| m.size).unwrap_or(0)),

                compressed_size: BigIntScalar(stats.compressed_size),
                compressed_size_range: history
                    .iter()
                    .map(|h| BigIntTimeSerie {
                        time: h.date,
                        value: BigIntScalar(h.compressed_size),
                    })
                    .collect(),
                compressed_size_last_month: BigIntScalar(
                    last_month.map(|e| e.compressed_size).unwrap_or(0),
                ),
            });
        }
        Ok(res)
    }
}

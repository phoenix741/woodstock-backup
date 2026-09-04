use async_graphql::ID;
use async_graphql::{Context, Object, Result as GqlResult};
use chrono::Local;
use tracing::debug;
use uuid::Uuid;

use crate::api::dto::{
    ApplicationEvent, ArchiveProfile, BigIntTimeSerie, DiskUsage as GqlDiskUsage, EventInformation,
    EventStep, EventsFilterInput, EventsPage, GqlStatistics, Host,
    HostStatistics as GqlHostStatistics, Job, MergedApplicationEvent, NumberTimeSerie,
    PoolHealthStatusDto, PoolUsage as GqlPoolUsage, QueueListInput, QueueStats,
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

        let raw_backups = state.backups.get_backups(&hostname).await;

        // Compute retention categories for the full list.
        let categories =
            super::types::compute_retention_categories(&hostname, state, &raw_backups).await;

        Ok(raw_backups
            .into_iter()
            .map(|b| {
                let id = b.id;
                let retention = categories.get(&id).copied().map(Into::into);
                BackupEx {
                    hostname: hostname.clone(),
                    inner: b.into(),
                    retention_category: retention,
                }
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
                        retention_category: None,
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
            retention_category: None,
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

        // There is no config field that stands in for "next wakeup" (see `bin/scheduler.rs`'s
        // module doc): the real value — the earliest due-date across every host, archive
        // profile, and nightly maintenance — is computed from live state inside the separate
        // scheduler process (`compute_next_wakeup`), which persists its last computed status
        // to Redis after every iteration (see `set_scanner_status`) precisely so this
        // resolver can read it back instead of guessing.
        let scanner_status =
            crate::jobs::scanner_status::get_scanner_status(&state.redis_client).await;
        let (last_execution, next_wakeup, next_wakeup_reason) = match scanner_status {
            Some(status) => (
                Some(status.last_execution),
                Some(status.next_wakeup),
                Some(status.next_wakeup_reason.into()),
            ),
            None => (None, None, None),
        };

        Ok(QueueStats {
            pending,
            running,
            success,
            failed,
            dead: 0, // not tracked in ProgressReader
            last_execution,
            next_wakeup,
            next_wakeup_reason,
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

    /// Configured archive profiles (from `archiving.yml`), read-only — used
    /// to populate the manual archive-run trigger in the Tasks UI. There is
    /// no mutation to create/edit profiles; that stays YAML-only.
    #[graphql(name = "archiveProfiles")]
    async fn archive_profiles(&self, ctx: &Context<'_>) -> GqlResult<Vec<ArchiveProfile>> {
        let state = ctx.data::<ApiServerState>()?;
        let archiving = woodstock::config::ArchivingConfig::new(state.config.clone());
        let profiles = archiving
            .list_profiles()
            .await
            .map_err(super::util::map_err)?;
        Ok(profiles.into_iter().map(Into::into).collect())
    }

    async fn events(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "firstEvent")] first_event: chrono::DateTime<Local>,
        #[graphql(name = "lastEvent")] last_event: chrono::DateTime<Local>,
        filter: Option<EventsFilterInput>,
        // Maximum number of events to return (default: 50, max: 500)
        limit: Option<i32>,
        // Pagination offset (default: 0)
        offset: Option<i32>,
    ) -> GqlResult<EventsPage> {
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
        let raw: Vec<ApplicationEvent> = events.into_iter().map(Into::into).collect();

        // Merge Start/End rows sharing the same uuid into one logical event, so
        // pagination/filtering operate on the unit the UI actually displays.
        let mut merged: std::collections::HashMap<String, MergedApplicationEvent> =
            std::collections::HashMap::new();
        for event in raw {
            let uuid = event.uuid.clone();
            let entry = merged
                .entry(uuid)
                .or_insert_with(|| MergedApplicationEvent {
                    uuid: event.uuid.clone(),
                    type_: event.type_,
                    source: event.source,
                    start_date: None,
                    end_date: None,
                    error_messages: Vec::new(),
                    status: event.status,
                    information: None,
                });
            match event.step {
                EventStep::Start => {
                    entry.start_date = Some(event.timestamp);
                    // Fallback so an in-progress event (no End row yet) still shows its info.
                    if entry.information.is_none() {
                        entry.information = event.information;
                        entry.status = event.status;
                        entry.error_messages = event.error_messages;
                    }
                }
                EventStep::End => {
                    entry.end_date = Some(event.timestamp);
                    entry.information = event.information;
                    entry.status = event.status;
                    entry.error_messages = event.error_messages;
                }
            }
        }

        let mut list: Vec<MergedApplicationEvent> = merged.into_values().collect();

        if let Some(filter) = filter.as_ref() {
            list.retain(|event| {
                if let Some(type_) = filter.type_ {
                    if event.type_ != type_ {
                        return false;
                    }
                }
                if let Some(status) = filter.status {
                    if event.status != status {
                        return false;
                    }
                }
                if let Some(source) = filter.source {
                    if event.source != source {
                        return false;
                    }
                }
                if let Some(hostname) = &filter.hostname {
                    let matches_hostname = matches!(
                        &event.information,
                        Some(EventInformation::EventBackupInformation(info)) if &info.hostname == hostname
                    );
                    if !matches_hostname {
                        return false;
                    }
                }
                true
            });
        }

        // End is always after Start when both are present, so it's the natural sort key;
        // fall back to Start for events that haven't ended yet.
        list.sort_by(|a, b| {
            let a_key = a.end_date.or(a.start_date);
            let b_key = b.end_date.or(b.start_date);
            b_key.cmp(&a_key)
        });

        // total_count reflects the merged, filtered unit the UI paginates over.
        let total_count = list.len() as i32;

        let skip = offset.unwrap_or(0).max(0) as usize;
        let take = limit.unwrap_or(50).clamp(1, 500) as usize;
        let items = list.into_iter().skip(skip).take(take).collect();

        Ok(EventsPage { items, total_count })
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

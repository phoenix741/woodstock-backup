//! Scheduler Binary: drives hosts, archive profiles, and nightly maintenance through a
//! single dynamic-wakeup loop plus an event-driven Pub/Sub subscriber (see
//! `run_scanner_loop`/`run_host_online_subscriber` below) — instead of separate Apalis
//! crons. Each category (host, archive profile, nightly) carries its own cron/period and
//! its own persisted "last run" state (backup timestamps for hosts, `ArchiveRunStatus` for
//! archive, a Redis key for nightly); the loop just computes, on every iteration, the
//! closest real deadline among the three and sleeps until then.
//!
//! Must run as a single instance: unlike `job_worker` (horizontally scalable via the Apalis
//! queues), `run_scanner_loop`/`run_host_online_subscriber` hold no distributed lock on the
//! scan/subscription itself — only the final enqueue is deduplicated
//! (`Producers::enqueue_backup_unique`, the archive/nightly job's name, and each category's
//! "last run" state). Multiple active instances at once duplicate scheduling work
//! (harmless but wasteful), never an actual backup.

use apalis_cron::Schedule;
use chrono::{DateTime, Duration as ChronoDuration, Local, TimeZone};
use color_eyre::eyre::Result;
use eyre::eyre;
use futures::StreamExt;
use redis::AsyncCommands;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};
use tokio::sync::Mutex;
use tracing::{info, warn};
use woodstock::archiving::ArchiveRunStatus;
use woodstock::config::{ArchivingConfig, Configuration, Scheduler, HOST_ONLINE_CHANNEL};
use woodstock::server::resolve::HostOnlineEvent;
use woodstock::utils::cron_due::next_due_at;
use woodstock_server_rs::{
    jobs::{
        decision::{
            get_next_attempt, set_next_attempt, try_schedule_host, SchedulingConfig,
            SchedulingOutcome, Trigger,
        },
        producers::Producers,
        scanner_status::{set_scanner_status, ScannerWakeupReason, ScannerWakeupReasonCategory},
        state::ApiWorkerState,
    },
    logger::init_logging,
};

/// Fallback safety-net ceiling used only if `wakeupSchedule` fails to parse as a cron
/// expression. Under normal operation the ceiling is the next tick of `wakeupSchedule`
/// itself (see [`compute_next_wakeup`]) — that field's role shifted from "the scanner's
/// fixed cadence" to "the safety-net upper bound for hosts that never self-register".
const WAKEUP_CEILING_FALLBACK_SECS: i64 = 3600;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    // Initialize crypto provider for rustls before any TLS operations
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| eyre!("Failed to install crypto provider"))?;

    let woodstock_config = Arc::new(Configuration::default());
    let scheduler_service = Scheduler::new(woodstock_config.clone());
    let scheduler = scheduler_service.get_schedule().await?;
    let scheduling_config = SchedulingConfig::from_application_scheduler(&scheduler);

    let state = Arc::new(ApiWorkerState::new(woodstock_config.clone()).await?);

    init_logging(&woodstock_config.path.logs_path, "scheduler.log")?;
    info!("Starting Scheduler (persisted cron to Redis)");

    let redis_url = woodstock_config.redis_url();
    info!("Connecting to Redis at: {}", redis_url);

    let archiving_config = Arc::new(ArchivingConfig::new(woodstock_config.clone()));

    // Producers to enqueue jobs when a host, archive profile, or nightly maintenance
    // is found due by the unified scanner loop below.
    let redis_client = redis::Client::open(redis_url.clone())?;
    let producers = Arc::new(Mutex::new(Producers::new(
        state.hosts.clone(),
        state.backups.clone(),
        state.apalis_redis_storage.schedule_storage.clone(),
        state.apalis_redis_storage.backup_storage.clone(),
        state.apalis_redis_storage.interactive_storage.clone(),
        state.apalis_redis_storage.maintenance_storage.clone(),
        state.apalis_redis_storage.archive_storage.clone(),
        state.progress_publisher.clone(),
        redis_client.clone(),
    )));

    // Listener of host-online events (transition offline -> online).
    // Publish from woodstock-rs/src/server/resolve.rs
    let subscriber_handle = tokio::spawn(run_host_online_subscriber(
        redis_client.clone(),
        state.clone(),
        producers.clone(),
        scheduling_config,
    ));

    // Unified scanner: runs the scheduling decision for every known host, every enabled
    // archive profile, and nightly maintenance, then sleeps until the next real deadline
    // across all three, computed by `compute_next_wakeup` — instead of ticking on a fixed
    // cron interval per category.
    let scanner_handle = tokio::spawn(run_scanner_loop(
        redis_client.clone(),
        state.clone(),
        producers.clone(),
        archiving_config.clone(),
        woodstock_config.path.jobs_path.clone(),
        scheduler.nightly_schedule.clone(),
        scheduler.wakeup_schedule.clone(),
        scheduling_config,
    ));

    // Both background tasks (`run_scanner_loop`, `run_host_online_subscriber`) loop
    // forever and only ever end via a panic — a panic in either is loud (process exit, so
    // a supervisor like systemd restarts a clean scheduler) instead of silently leaving
    // the scheduler running with no scheduling.
    tokio::select! {
        res = scanner_handle => {
            return Err(match res {
                Ok(()) => eyre!("Scanner loop task ended unexpectedly"),
                Err(e) => eyre!("Scanner loop task panicked: {e}"),
            });
        }
        res = subscriber_handle => {
            return Err(match res {
                Ok(()) => eyre!("Host-online subscriber task ended unexpectedly"),
                Err(e) => eyre!("Host-online subscriber task panicked: {e}"),
            });
        }
    }
}

/// Subscribes to [`HOST_ONLINE_CHANNEL`] and immediately runs the scheduling decision for
/// any host that just transitioned from offline to online (see
/// `SocketAddrResolver::register_service`), instead of waiting for the scanner's next
/// dynamic wakeup. Runs forever: any Redis/connection error just logs a warning and
/// retries after a short delay — this path is a responsiveness improvement on top of the
/// scanner loop's safety net, never the only way a host gets backed up.
async fn run_host_online_subscriber(
    redis_client: redis::Client,
    state: Arc<ApiWorkerState>,
    producers: Arc<Mutex<Producers>>,
    scheduling_config: SchedulingConfig,
) {
    loop {
        let mut pubsub = match redis_client.get_async_pubsub().await {
            Ok(pubsub) => pubsub,
            Err(e) => {
                warn!("Failed to open Redis pubsub for {HOST_ONLINE_CHANNEL}: {e}");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        };

        if let Err(e) = pubsub.subscribe(HOST_ONLINE_CHANNEL).await {
            warn!("Failed to subscribe to {HOST_ONLINE_CHANNEL}: {e}");
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            continue;
        }
        info!("Subscribed to {HOST_ONLINE_CHANNEL} for event-driven backups");

        let mut messages = pubsub.into_on_message();
        while let Some(msg) = messages.next().await {
            let payload: String = match msg.get_payload() {
                Ok(payload) => payload,
                Err(e) => {
                    warn!("Bad payload on {HOST_ONLINE_CHANNEL}: {e}");
                    continue;
                }
            };
            let event: HostOnlineEvent = match serde_json::from_str(&payload) {
                Ok(event) => event,
                Err(e) => {
                    warn!("Failed to parse HostOnlineEvent from {HOST_ONLINE_CHANNEL}: {e}");
                    continue;
                }
            };

            info!(
                "Host {} just came online, checking for a due backup",
                event.hostname
            );
            // Bypass any recorded cooldown: the host coming online is exactly the new
            // information that invalidates a previous `SkippedUnreachable` backoff.
            match try_schedule_host(
                &state.job_utility,
                &producers,
                &redis_client,
                &event.hostname,
                false,
                Trigger::OnlineEvent,
                &scheduling_config,
            )
            .await
            {
                Ok(outcome) => {
                    info!(
                        "Event-driven scheduling for {}: {:?}",
                        event.hostname, outcome
                    )
                }
                Err(e) => warn!("Event-driven scheduling failed for {}: {e}", event.hostname),
            }
        }

        // The pubsub stream ended (connection dropped, or the server closed a subscriber
        // it still accepted — e.g. hitting maxclients, or a proxy dropping idle
        // subscribers): loop back and resubscribe, with the same short delay as the
        // connect/subscribe error paths above, so a server that keeps closing the
        // subscription right after accepting it can't turn this into a tight busy-loop.
        warn!("Redis pubsub stream for {HOST_ONLINE_CHANNEL} ended, reconnecting");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

/// Redis key holding the nightly maintenance trigger's last-run timestamp — the nightly
/// equivalent of [`ArchiveRunStatus`]. Needed because the unified scanner loop no longer
/// relies on Apalis-cron's own persisted-tick replay to survive a restart: due-ness is
/// instead recomputed from this timestamp plus `nightlySchedule` on every iteration, via
/// [`next_due_at`] — the same mechanism already used for archive profiles and, in spirit,
/// for hosts (real persisted completion state, not tick-stream replay).
const NIGHTLY_LAST_RUN_KEY: &str = "woodstock:schedule:nightly:last-run";

/// Cooldown identifier for nightly maintenance, passed to [`get_next_attempt`]/
/// [`set_next_attempt`] — the same "next attempt allowed" store hosts use, under a
/// distinct, non-hostname-shaped key so a failure here backs off instead of retrying on
/// every scanner iteration (see `check_and_enqueue_nightly`).
const NIGHTLY_ATTEMPT_KEY: &str = "nightly-maintenance";

/// Cooldown identifier for one archive profile, passed to [`get_next_attempt`]/
/// [`set_next_attempt`] — see [`NIGHTLY_ATTEMPT_KEY`].
fn archive_profile_attempt_key(profile_name: &str) -> String {
    format!("archive-profile:{profile_name}")
}

async fn get_nightly_last_run(redis_client: &redis::Client) -> Option<DateTime<Local>> {
    let mut con = match redis_client.get_multiplexed_async_connection().await {
        Ok(con) => con,
        Err(e) => {
            warn!("Failed to get Redis connection to read nightly last-run: {e}");
            return None;
        }
    };
    let ts: Option<i64> = match con.get(NIGHTLY_LAST_RUN_KEY).await {
        Ok(ts) => ts,
        Err(e) => {
            warn!("Failed to read nightly last-run: {e}");
            return None;
        }
    };
    ts.and_then(|ts| Local.timestamp_opt(ts, 0).single())
}

async fn set_nightly_last_run(redis_client: &redis::Client, at: DateTime<Local>) {
    let Ok(mut con) = redis_client.get_multiplexed_async_connection().await else {
        warn!("Failed to get Redis connection to persist nightly last-run");
        return;
    };
    if let Err(e) = con
        .set::<_, _, ()>(NIGHTLY_LAST_RUN_KEY, at.timestamp())
        .await
    {
        warn!("Failed to persist nightly last-run: {e}");
    }
}

/// Checks every enabled archive profile's due-ness and enqueues a run for those that are
/// due — the scanner-loop equivalent of the former `cron-archive` Apalis worker's body,
/// now driven by [`next_due_at`] instead of a fixed 5-minute tick.
///
/// A failure to enqueue, or to persist `last_run` after a successful enqueue, records a
/// cooldown (mirroring `try_schedule_host`'s refusal backoff for hosts) instead of leaving
/// the profile permanently "due": without it, the next scanner iteration would recompute
/// the exact same due-ness and retry as fast as `wakeup_floor_secs` until the underlying
/// failure clears.
async fn check_and_enqueue_due_archives(
    archiving_config: &ArchivingConfig,
    jobs_path: &Path,
    producers: &Arc<Mutex<Producers>>,
    redis_client: &redis::Client,
    scheduling_config: &SchedulingConfig,
    now: DateTime<Local>,
) {
    let profiles = match archiving_config.list_profiles().await {
        Ok(profiles) => profiles,
        Err(e) => {
            warn!("Scanner: failed to load archiving.yml: {e}");
            return;
        }
    };

    for profile in profiles {
        if !profile.enabled {
            continue;
        }

        let attempt_key = archive_profile_attempt_key(&profile.name);
        if let Some(retry_at) = get_next_attempt(redis_client, &attempt_key).await {
            if retry_at > now {
                continue;
            }
        }

        let status = ArchiveRunStatus::load(jobs_path, &profile.name)
            .await
            .unwrap_or_default();

        let due = match next_due_at(&profile.schedule_cron, status.last_run, now) {
            Ok(Some(due_at)) => due_at <= now,
            Ok(None) => false,
            Err(e) => {
                warn!("Scanner: skipping archive profile '{}': {e}", profile.name);
                continue;
            }
        };

        if !due {
            continue;
        }

        let mut prod = producers.lock().await;
        match prod
            .enqueue_archive_profile(archiving_config, &profile.name)
            .await
        {
            Ok(job_ids) => {
                if job_ids.is_empty() {
                    info!(
                        "Archive profile '{}' due: no host matched its selection, nothing enqueued",
                        profile.name
                    );
                } else {
                    info!("Archive profile '{}' due: enqueued 1 job", profile.name);
                }
                let new_status = ArchiveRunStatus {
                    last_run: Some(now),
                };
                if let Err(e) = new_status.save(jobs_path, &profile.name).await {
                    tracing::error!(
                        "Failed to persist run status for archive profile '{}': {e}",
                        profile.name
                    );
                    set_next_attempt(
                        redis_client,
                        &attempt_key,
                        now + ChronoDuration::seconds(
                            scheduling_config.retry_backoff_on_refusal_secs,
                        ),
                        scheduling_config,
                    )
                    .await;
                }
            }
            Err(e) => {
                tracing::error!("Failed to enqueue archive profile '{}': {e}", profile.name);
                set_next_attempt(
                    redis_client,
                    &attempt_key,
                    now + ChronoDuration::seconds(scheduling_config.retry_backoff_on_refusal_secs),
                    scheduling_config,
                )
                .await;
            }
        }
    }
}

/// Checks the nightly maintenance trigger's due-ness (via [`next_due_at`] against
/// [`NIGHTLY_LAST_RUN_KEY`]) and enqueues `CleanupRefcnt` if due — the scanner-loop
/// equivalent of the former `cron-nightly` Apalis worker's body.
///
/// A failed enqueue records a cooldown under [`NIGHTLY_ATTEMPT_KEY`] instead of leaving the
/// trigger permanently "due" — see [`check_and_enqueue_due_archives`]'s doc for why.
async fn check_and_enqueue_nightly(
    redis_client: &redis::Client,
    producers: &Arc<Mutex<Producers>>,
    nightly_schedule: &str,
    scheduling_config: &SchedulingConfig,
    now: DateTime<Local>,
) {
    if let Some(retry_at) = get_next_attempt(redis_client, NIGHTLY_ATTEMPT_KEY).await {
        if retry_at > now {
            return;
        }
    }

    let last_run = get_nightly_last_run(redis_client).await;
    let due = match next_due_at(nightly_schedule, last_run, now) {
        Ok(Some(due_at)) => due_at <= now,
        Ok(None) => false,
        Err(e) => {
            warn!("Scanner: invalid nightlySchedule cron expression: {e}");
            false
        }
    };
    if !due {
        return;
    }

    let mut prod = producers.lock().await;
    match prod.enqueue_cleanup_refcnt().await {
        Ok(_) => {
            info!("Nightly cleanup refcnt job enqueued");
            drop(prod);
            set_nightly_last_run(redis_client, now).await;
        }
        Err(e) => {
            tracing::error!("Failed to enqueue nightly cleanup refcnt: {e}");
            set_next_attempt(
                redis_client,
                NIGHTLY_ATTEMPT_KEY,
                now + ChronoDuration::seconds(scheduling_config.retry_backoff_on_refusal_secs),
                scheduling_config,
            )
            .await;
        }
    }
}

/// Unified safety-net scanner: on each iteration, runs the scheduling decision for every
/// known host, checks every enabled archive profile and the nightly maintenance trigger
/// for due-ness, then sleeps until the next real deadline across all three, computed by
/// [`compute_next_wakeup`] — instead of ticking on a fixed cron interval per category.
async fn run_scanner_loop(
    redis_client: redis::Client,
    state: Arc<ApiWorkerState>,
    producers: Arc<Mutex<Producers>>,
    archiving_config: Arc<ArchivingConfig>,
    jobs_path: PathBuf,
    nightly_schedule: String,
    wakeup_ceiling_cron: String,
    scheduling_config: SchedulingConfig,
) {
    loop {
        // A failure here only prevents *host* scheduling this iteration — archive
        // profiles and nightly maintenance don't depend on the host list, so they still
        // get checked below instead of being delayed by an unrelated failure.
        let (hosts, hosts_list_ok) = match state.hosts.list_hosts().await {
            Ok(hosts) => (hosts, true),
            Err(e) => {
                warn!("Scanner: failed to list hosts: {e}");
                (Vec::new(), false)
            }
        };

        if hosts_list_ok {
            for host in &hosts {
                match try_schedule_host(
                    &state.job_utility,
                    &producers,
                    &redis_client,
                    host,
                    false,
                    Trigger::Scan,
                    &scheduling_config,
                )
                .await
                {
                    Ok(SchedulingOutcome::NotDue | SchedulingOutcome::SkippedCooldown { .. }) => {}
                    Ok(outcome) => info!("Scanner: {host} -> {outcome:?}"),
                    Err(e) => warn!("Scanner: scheduling failed for {host}: {e}"),
                }
            }
        }

        let now = Local::now();
        check_and_enqueue_due_archives(
            &archiving_config,
            &jobs_path,
            &producers,
            &redis_client,
            &scheduling_config,
            now,
        )
        .await;
        check_and_enqueue_nightly(
            &redis_client,
            &producers,
            &nightly_schedule,
            &scheduling_config,
            now,
        )
        .await;

        let (mut next_wakeup, mut reason) = compute_next_wakeup(
            &state,
            &redis_client,
            &hosts,
            &archiving_config,
            &jobs_path,
            &nightly_schedule,
            &wakeup_ceiling_cron,
            &scheduling_config,
        )
        .await;

        // `hosts` was empty because listing failed, not because there are genuinely no
        // hosts — don't let that empty list push the computed wakeup out to the ceiling
        // (up to an hour); retry listing at the usual floor instead.
        if !hosts_list_ok {
            let retry_floor =
                Local::now() + ChronoDuration::seconds(scheduling_config.wakeup_floor_secs);
            if retry_floor < next_wakeup {
                next_wakeup = retry_floor;
                reason = ScannerWakeupReason::without_subject(
                    ScannerWakeupReasonCategory::HostListUnavailable,
                );
            }
        }

        let sleep_for =
            (next_wakeup - Local::now())
                .to_std()
                .unwrap_or(std::time::Duration::from_secs(
                    scheduling_config.wakeup_floor_secs as u64,
                ));
        info!("Scanner: sleeping {sleep_for:?}, next wakeup at {next_wakeup} ({reason})");
        // Persist the same status just logged, so `queueStats` (a separate process) can
        // report it instead of re-running this computation without access to the live
        // state (Redis cooldowns, per-host schedules, archive profiles) it depends on.
        set_scanner_status(&redis_client, now, next_wakeup, &reason).await;
        tokio::time::sleep(sleep_for).await;
    }
}

/// Computes when the scanner should next wake up: the earliest, across all hosts, enabled
/// archive profiles, and the nightly maintenance trigger, of "when it's next due" (bumped
/// forward by any recorded backoff for hosts). Each category's due-date is a real calendar
/// date computed from its own persisted "last done" state (see `JobUtility::get_time_to_next_backup`
/// for hosts, [`next_due_at`] for archive profiles/nightly), not an estimate — so there is
/// nothing to gain by waking up any earlier than that: for a host that isn't due yet
/// `should_backup_host` would just refuse (it runs *before* any reachability check, so a
/// too-early wake doesn't even ping the host), and for archive/nightly a too-early wake
/// finds nothing due either.
///
/// `wakeup_ceiling_cron` is applied once, globally, as a final safety-net cap on the result
/// — not per category — for the one case actual due-dates can't cover: a config change (new
/// host, re-activated schedule, shortened `backupPeriod`, a new/edited archive profile, an
/// edited `nightlySchedule`) made between now and the computed wakeup, which the scanner can
/// only notice by waking up and re-reading configuration.
async fn compute_next_wakeup(
    state: &ApiWorkerState,
    redis_client: &redis::Client,
    hosts: &[String],
    archiving_config: &ArchivingConfig,
    jobs_path: &Path,
    nightly_schedule: &str,
    wakeup_ceiling_cron: &str,
    scheduling_config: &SchedulingConfig,
) -> (DateTime<Local>, ScannerWakeupReason) {
    use ScannerWakeupReasonCategory::*;

    let now = Local::now();
    let floor = now + ChronoDuration::seconds(scheduling_config.wakeup_floor_secs);
    let ceiling_dt = match Schedule::from_str(wakeup_ceiling_cron) {
        Ok(schedule) => schedule
            .after(&now)
            .next()
            .unwrap_or(now + ChronoDuration::seconds(WAKEUP_CEILING_FALLBACK_SECS)),
        Err(e) => {
            warn!(
                "Invalid wakeupSchedule cron expression {wakeup_ceiling_cron:?} ({e}), falling back to a {WAKEUP_CEILING_FALLBACK_SECS}s ceiling"
            );
            now + ChronoDuration::seconds(WAKEUP_CEILING_FALLBACK_SECS)
        }
    };

    let mut winner: Option<(DateTime<Local>, ScannerWakeupReason)> = None;

    for host in hosts {
        let due = match state.job_utility.get_time_to_next_backup(host).await {
            Ok(Some(due)) => due,
            Ok(None) => continue, // schedule not activated for this host
            Err(e) => {
                warn!("Scanner: failed to compute time-to-next-backup for {host}: {e}");
                continue;
            }
        };

        let mut candidate = now + due;
        let mut reason = ScannerWakeupReason::new(HostDue, host.clone());
        if let Some(next_attempt) = get_next_attempt(redis_client, host).await {
            if next_attempt > candidate {
                candidate = next_attempt;
                reason = ScannerWakeupReason::new(HostBackoff, host.clone());
            }
        }

        if candidate < floor {
            candidate = floor;
            reason = ScannerWakeupReason::new(HostCappedByFloor, host.clone());
        }

        if winner.as_ref().is_none_or(|(cur, _)| candidate < *cur) {
            winner = Some((candidate, reason));
        }
    }

    match archiving_config.list_profiles().await {
        Ok(profiles) => {
            for profile in profiles {
                if !profile.enabled {
                    continue;
                }
                let status = ArchiveRunStatus::load(jobs_path, &profile.name)
                    .await
                    .unwrap_or_default();
                let due_at = match next_due_at(&profile.schedule_cron, status.last_run, now) {
                    Ok(Some(due_at)) => due_at,
                    Ok(None) => continue,
                    Err(e) => {
                        warn!(
                            "Scanner: invalid schedule for archive profile '{}': {e}",
                            profile.name
                        );
                        continue;
                    }
                };

                let mut candidate = due_at;
                let mut reason = ScannerWakeupReason::new(ArchiveProfileDue, profile.name.clone());
                let attempt_key = archive_profile_attempt_key(&profile.name);
                if let Some(next_attempt) = get_next_attempt(redis_client, &attempt_key).await {
                    if next_attempt > candidate {
                        candidate = next_attempt;
                        reason =
                            ScannerWakeupReason::new(ArchiveProfileBackoff, profile.name.clone());
                    }
                }
                if candidate < floor {
                    candidate = floor;
                    reason =
                        ScannerWakeupReason::new(ArchiveProfileCappedByFloor, profile.name.clone());
                }
                if winner.as_ref().is_none_or(|(cur, _)| candidate < *cur) {
                    winner = Some((candidate, reason));
                }
            }
        }
        Err(e) => warn!("Scanner: failed to load archiving.yml: {e}"),
    }

    let nightly_last_run = get_nightly_last_run(redis_client).await;
    match next_due_at(nightly_schedule, nightly_last_run, now) {
        Ok(Some(due_at)) => {
            let mut candidate = due_at;
            let mut reason = ScannerWakeupReason::without_subject(NightlyDue);
            if let Some(next_attempt) = get_next_attempt(redis_client, NIGHTLY_ATTEMPT_KEY).await {
                if next_attempt > candidate {
                    candidate = next_attempt;
                    reason = ScannerWakeupReason::without_subject(NightlyBackoff);
                }
            }
            if candidate < floor {
                candidate = floor;
                reason = ScannerWakeupReason::without_subject(NightlyCappedByFloor);
            }
            if winner.as_ref().is_none_or(|(cur, _)| candidate < *cur) {
                winner = Some((candidate, reason));
            }
        }
        Ok(None) => {}
        Err(e) => warn!("Scanner: invalid nightlySchedule cron expression: {e}"),
    }

    match winner {
        None => (
            ceiling_dt.max(floor),
            ScannerWakeupReason::without_subject(NothingDueCeiling),
        ),
        Some((candidate, _reason)) if candidate > ceiling_dt => (
            ceiling_dt.max(floor),
            ScannerWakeupReason::without_subject(SafetyCeiling),
        ),
        Some(winner) => winner,
    }
}

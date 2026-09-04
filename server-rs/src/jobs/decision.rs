//! Shared scheduling decision, used by both the periodic scanner loop and the
//! event-driven "host just came online" subscriber in `bin/scheduler.rs` — so the
//! two triggers can never diverge on whether a given host may be backed up right now.

use std::sync::Arc;

use chrono::{DateTime, Duration, Local, TimeZone};
use redis::AsyncCommands;
use tokio::sync::Mutex;
use tracing::{debug, warn};
use woodstock::config::ApplicationScheduler;
use woodstock::server::job::JobUtility;

use crate::jobs::producers::Producers;

/// Floor under the "next attempt allowed" Redis key's TTL, so a very short configured
/// backoff doesn't leave the key expiring implausibly fast — kept at the old hardcoded
/// TTL's value so behavior at default settings is unchanged.
const NEXT_ATTEMPT_TTL_FLOOR_SECS: i64 = 24 * 3600;

/// Scheduling cadence, sourced from `ApplicationScheduler` (`scheduler.yml`).
#[derive(Debug, Clone, Copy)]
pub struct SchedulingConfig {
    /// Floor under the scanner's dynamic sleep, so a host/profile/nightly trigger stuck
    /// permanently "due" can never turn the loop into a busy-poll. See `bin/scheduler.rs`.
    pub wakeup_floor_secs: i64,
    /// Cooldown recorded after a *successful* enqueue: long enough that the dynamic-wakeup
    /// computation (see `bin/scheduler.rs`) doesn't immediately re-consider this host again
    /// before the job it just enqueued has had a chance to start. Distinct from
    /// `Producers::enqueue_backup_unique`'s own 30s Redis dedup key, which only protects
    /// against a double-enqueue race within the same instant.
    pub retry_backoff_after_success_secs: i64,
    /// Cooldown recorded after a refused attempt (host unreachable, already running, or
    /// blocked by an active pool fsck lock — blackout uses its own computed `retry_at`
    /// instead, see [`woodstock::server::job::JobUtility::is_in_blackout_now`]). Defaults to
    /// the *old* fixed scanner cadence (15 min), not shorter: an offline-and-due host is the
    /// single most common steady state of this scheduler (an idle laptop), and a host coming
    /// back online is already covered instantly by the event-driven path (`bypass_cooldown`)
    /// — so polling it more often than that by default would make the rework *less*
    /// efficient in exactly its most common case, not more.
    pub retry_backoff_on_refusal_secs: i64,
}

impl SchedulingConfig {
    #[must_use]
    pub fn from_application_scheduler(scheduler: &ApplicationScheduler) -> Self {
        Self {
            wakeup_floor_secs: scheduler.wakeup_floor_secs,
            retry_backoff_after_success_secs: scheduler.retry_backoff_after_success_secs,
            retry_backoff_on_refusal_secs: scheduler.retry_backoff_on_refusal_secs,
        }
    }

    /// TTL of the "next attempt allowed" Redis key: generous enough to survive between two
    /// attempts even with a large configured backoff. Keeps the old fixed 24h as a floor
    /// (so behavior at default settings is unchanged) rather than a hard ceiling that a
    /// large custom backoff could silently exceed.
    fn next_attempt_ttl_secs(&self) -> i64 {
        (self
            .retry_backoff_after_success_secs
            .max(self.retry_backoff_on_refusal_secs)
            * 2)
        .max(NEXT_ATTEMPT_TTL_FLOOR_SECS)
    }
}

/// Key under which a "next attempt allowed" cooldown is recorded. Despite the `host`
/// parameter name (the overwhelming majority of callers), this is really keyed on any
/// scanner-loop item identifier — `bin/scheduler.rs` reuses the same cooldown store for
/// archive profiles and nightly maintenance under their own distinct, non-hostname-shaped
/// identifiers (`archive-profile:<name>`, `nightly-maintenance`), so the same
/// busy-poll-on-repeated-failure protection applies to every category the scanner drives,
/// not just hosts.
fn next_attempt_key(host: &str) -> String {
    format!("woodstock:schedule:next-attempt:{host}")
}

/// Outcome of a single [`try_schedule_host`] call, mirroring the gates it checks in order.
#[derive(Debug, Clone, PartialEq)]
pub enum SchedulingOutcome {
    Enqueued(String),
    /// A cooldown recorded by a previous attempt is still in effect (see
    /// [`get_next_attempt`]) — none of the other gates were even checked.
    SkippedCooldown {
        retry_at: DateTime<Local>,
    },
    SkippedRunning,
    SkippedBlockedByFsck,
    SkippedUnreachable,
    SkippedBlackout {
        retry_at: DateTime<Local>,
    },
    NotDue,
}

/// Where a [`try_schedule_host`] call originates from — decides whether a cooldown
/// recorded by a previous attempt is honored. A plain `bool` here would sit right next to
/// `force` with no type-level distinction between the two, so this stays a dedicated enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    /// The periodic safety-net scan — always honors any recorded cooldown.
    Scan,
    /// The event-driven "host just came online" path — the host coming online is exactly
    /// the new information that invalidates a cooldown recorded by a previous
    /// `SkippedUnreachable`, so this bypasses it.
    ///
    /// Known race, accepted as harmless: bypassing the cooldown only guards against
    /// re-enqueuing a *stuck* host, not a host whose job is merely queued but not yet
    /// started (`is_job_running` only becomes true once a worker acquires the pool lock,
    /// and `enqueue_backup_unique`'s own dedup key is a short 30s window). A host flapping
    /// online again while `job_worker` is backlogged can therefore get a second job enqueued
    /// for the same backup. This produces a redundant queue entry, not a corrupted backup —
    /// the pool's exclusive per-host lock still serializes the two jobs at execution time.
    OnlineEvent,
}

/// Runs the full scheduling decision for `host` and enqueues a backup job if every gate
/// passes: not on cooldown, not already running, due for a backup, not in an unoverridden
/// blackout window, not blocked by a pool fsck lock, and reachable.
///
/// Every refusal (and success) is recorded in the per-host "next attempt allowed" Redis
/// key, both to gate the *next* call to this function (see `trigger` below) and to feed the
/// scanner's dynamic-wakeup computation, so a host stuck refusing (unreachable, still
/// running, in blackout) is retried on its own backoff instead of on every scan.
pub async fn try_schedule_host(
    job_utility: &JobUtility,
    producers: &Arc<Mutex<Producers>>,
    redis_client: &redis::Client,
    host: &str,
    force: bool,
    trigger: Trigger,
    config: &SchedulingConfig,
) -> eyre::Result<SchedulingOutcome> {
    if trigger == Trigger::Scan {
        if let Some(retry_at) = get_next_attempt(redis_client, host).await {
            if retry_at > Local::now() {
                return Ok(SchedulingOutcome::SkippedCooldown { retry_at });
            }
        }
    }

    if job_utility.is_job_running(host).await? {
        debug!("Skipping host {host}: job already running");
        set_next_attempt(
            redis_client,
            host,
            Local::now() + Duration::seconds(config.retry_backoff_on_refusal_secs),
            config,
        )
        .await;
        return Ok(SchedulingOutcome::SkippedRunning);
    }

    if !job_utility.should_backup_host(host, force).await? {
        return Ok(SchedulingOutcome::NotDue);
    }

    if let Some(retry_at) = job_utility.is_in_blackout_now(host).await? {
        set_next_attempt(redis_client, host, retry_at, config).await;
        return Ok(SchedulingOutcome::SkippedBlackout { retry_at });
    }

    if !job_utility.can_launch_backup(host).await? {
        debug!("Skipping host {host}: pool fsck lock is active");
        set_next_attempt(
            redis_client,
            host,
            Local::now() + Duration::seconds(config.retry_backoff_on_refusal_secs),
            config,
        )
        .await;
        return Ok(SchedulingOutcome::SkippedBlockedByFsck);
    }

    if !job_utility.host_available(host).await? {
        set_next_attempt(
            redis_client,
            host,
            Local::now() + Duration::seconds(config.retry_backoff_on_refusal_secs),
            config,
        )
        .await;
        return Ok(SchedulingOutcome::SkippedUnreachable);
    }

    let job_id = {
        let mut prod = producers.lock().await;
        prod.enqueue_backup_unique(host, force)
            .await
            .map_err(|e| eyre::eyre!("Failed to enqueue backup for host {host}: {e}"))?
    };

    match job_id {
        Some(job_id) => {
            set_next_attempt(
                redis_client,
                host,
                Local::now() + Duration::seconds(config.retry_backoff_after_success_secs),
                config,
            )
            .await;
            Ok(SchedulingOutcome::Enqueued(job_id))
        }
        // A backup for this host was already enqueued a moment ago (30s dedup key).
        None => Ok(SchedulingOutcome::NotDue),
    }
}

pub async fn set_next_attempt(
    redis_client: &redis::Client,
    host: &str,
    at: DateTime<Local>,
    config: &SchedulingConfig,
) {
    let Ok(mut con) = redis_client.get_multiplexed_async_connection().await else {
        warn!("Failed to get Redis connection to persist next-attempt for host {host}");
        return;
    };
    let key = next_attempt_key(host);
    if let Err(e) = con
        .set_ex::<_, _, ()>(&key, at.timestamp(), config.next_attempt_ttl_secs() as u64)
        .await
    {
        warn!("Failed to persist next-attempt for host {host}: {e}");
    }
}

/// Reads the "next attempt allowed" timestamp previously recorded for `host` by
/// [`try_schedule_host`], if any. `None` means no cooldown is in effect (either the host
/// has never been attempted, or the key has expired) — but also, on a Redis failure, since
/// a missing cooldown only ever makes this function let a host through *early*, never blocks
/// it, which is the safer failure mode. Redis errors are logged so a systemic outage (which
/// would otherwise look like every host's cooldown mysteriously disappearing) is visible.
pub async fn get_next_attempt(redis_client: &redis::Client, host: &str) -> Option<DateTime<Local>> {
    let mut con = match redis_client.get_multiplexed_async_connection().await {
        Ok(con) => con,
        Err(e) => {
            warn!("Failed to get Redis connection to read next-attempt for host {host}: {e}");
            return None;
        }
    };
    let ts: Option<i64> = match con.get(next_attempt_key(host)).await {
        Ok(ts) => ts,
        Err(e) => {
            warn!("Failed to read next-attempt for host {host}: {e}");
            return None;
        }
    };
    ts.and_then(|ts| Local.timestamp_opt(ts, 0).single())
}

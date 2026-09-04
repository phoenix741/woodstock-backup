//! Redis-persisted snapshot of the scanner loop's last completed iteration and its computed
//! next wakeup, so any process (the API server's `queueStats` GraphQL resolver) can report
//! the same status `bin/scheduler.rs` already logs every iteration ("Scanner: sleeping Xs,
//! next wakeup at Y (reason)") without re-running the scheduling computation itself — that
//! computation depends on live state (Redis cooldowns, per-host schedules, archive profiles)
//! only the scheduler process reads directly.

use std::collections::HashMap;
use std::fmt;

use chrono::{DateTime, Local, TimeZone};
use redis::AsyncCommands;
use tracing::warn;

const SCANNER_STATUS_KEY: &str = "woodstock:schedule:scanner-status";

/// Grace margin added on top of `next_wakeup - last_execution` (see [`status_ttl_secs`]) so
/// the entry doesn't expire right as the scanner is about to legitimately rewrite it.
const SCANNER_STATUS_TTL_GRACE_SECS: i64 = 3600;

/// Floor under the computed TTL, for the (degenerate, near-instant) case where `next_wakeup`
/// is essentially now — still leaves the grace margin's worth of slack.
const SCANNER_STATUS_TTL_FLOOR_SECS: i64 = SCANNER_STATUS_TTL_GRACE_SECS;

/// How long a status snapshot stays valid before reading back as unknown, computed per call
/// from how far out this iteration's `next_wakeup` actually is — unlike a fixed TTL, this
/// scales with a scanner that's genuinely, legitimately asleep for a long time (a host not
/// due for days is not a down scheduler) instead of misreporting it as one. No update by the
/// time this elapses means the scheduler process itself is down, not just idle.
fn status_ttl_secs(last_execution: DateTime<Local>, next_wakeup: DateTime<Local>) -> i64 {
    let sleep_span = (next_wakeup - last_execution).num_seconds().max(0);
    (sleep_span + SCANNER_STATUS_TTL_GRACE_SECS).max(SCANNER_STATUS_TTL_FLOOR_SECS)
}

/// Category of [`ScannerWakeupReason`] — deliberately a plain enum with no attached data, so
/// it maps 1:1 onto a GraphQL enum (`api::dto::queue::ScannerWakeupReasonCategory`) a future
/// translation layer can key off of, instead of a free-text string nobody can localize. Any
/// variable part (which host, which archive profile) travels separately in
/// [`ScannerWakeupReason::subject`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScannerWakeupReasonCategory {
    HostDue,
    HostBackoff,
    HostCappedByFloor,
    ArchiveProfileDue,
    ArchiveProfileBackoff,
    ArchiveProfileCappedByFloor,
    NightlyDue,
    NightlyBackoff,
    NightlyCappedByFloor,
    /// Nothing is due anywhere (no active host schedule, no enabled archive profile, nightly
    /// disabled) — see `bin/scheduler.rs`'s `NO_WORK_FALLBACK_SECS`.
    NothingDue,
    /// At least one host is due but known offline in the resolver cache, so it was excluded
    /// from the wakeup computation entirely — it's waiting on its own online event, not a
    /// timer. `subject` carries the affected hostname(s), comma-separated if more than one.
    OnlineEventPending,
    HostListUnavailable,
}

impl ScannerWakeupReasonCategory {
    /// Stable snake_case tag persisted in Redis — not `Display` (that's reserved for the
    /// human-readable log phrasing on [`ScannerWakeupReason`]) and not `serde` (this value is
    /// never serialized as part of a larger structure, only ever written/read as one hash
    /// field, so a derive would be dead surface).
    fn as_str(self) -> &'static str {
        match self {
            Self::HostDue => "host_due",
            Self::HostBackoff => "host_backoff",
            Self::HostCappedByFloor => "host_capped_by_floor",
            Self::ArchiveProfileDue => "archive_profile_due",
            Self::ArchiveProfileBackoff => "archive_profile_backoff",
            Self::ArchiveProfileCappedByFloor => "archive_profile_capped_by_floor",
            Self::NightlyDue => "nightly_due",
            Self::NightlyBackoff => "nightly_backoff",
            Self::NightlyCappedByFloor => "nightly_capped_by_floor",
            Self::NothingDue => "nothing_due",
            Self::OnlineEventPending => "online_event_pending",
            Self::HostListUnavailable => "host_list_unavailable",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "host_due" => Self::HostDue,
            "host_backoff" => Self::HostBackoff,
            "host_capped_by_floor" => Self::HostCappedByFloor,
            "archive_profile_due" => Self::ArchiveProfileDue,
            "archive_profile_backoff" => Self::ArchiveProfileBackoff,
            "archive_profile_capped_by_floor" => Self::ArchiveProfileCappedByFloor,
            "nightly_due" => Self::NightlyDue,
            "nightly_backoff" => Self::NightlyBackoff,
            "nightly_capped_by_floor" => Self::NightlyCappedByFloor,
            "nothing_due" => Self::NothingDue,
            "online_event_pending" => Self::OnlineEventPending,
            "host_list_unavailable" => Self::HostListUnavailable,
            _ => return None,
        })
    }
}

/// Why the scanner computed the `next_wakeup` it did — a [`ScannerWakeupReasonCategory`] plus,
/// for the categories about one specific host or archive profile, its name. Kept structured
/// (instead of a formatted string) so a GraphQL client can translate `category` and interpolate
/// `subject`, rather than parsing English prose.
#[derive(Debug, Clone)]
pub struct ScannerWakeupReason {
    pub category: ScannerWakeupReasonCategory,
    pub subject: Option<String>,
}

impl ScannerWakeupReason {
    pub fn new(category: ScannerWakeupReasonCategory, subject: impl Into<String>) -> Self {
        Self {
            category,
            subject: Some(subject.into()),
        }
    }

    pub fn without_subject(category: ScannerWakeupReasonCategory) -> Self {
        Self {
            category,
            subject: None,
        }
    }
}

/// Human-readable phrasing for the scheduler's own `info!` log line — the *only* place this
/// text is used; the GraphQL-facing data stays structured (`category` + `subject`) precisely
/// so it doesn't need to be parsed back out of prose like this. Deliberately doesn't repeat the
/// due date/backoff timestamp: that's already logged right next to it as `next_wakeup`.
impl fmt::Display for ScannerWakeupReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ScannerWakeupReasonCategory::*;
        match (self.category, self.subject.as_deref()) {
            (HostDue, Some(host)) => write!(f, "{host} due"),
            (HostBackoff, Some(host)) => write!(f, "{host} backoff"),
            (HostCappedByFloor, Some(host)) => write!(f, "{host} capped by wakeup floor"),
            (ArchiveProfileDue, Some(profile)) => write!(f, "archive profile '{profile}' due"),
            (ArchiveProfileBackoff, Some(profile)) => {
                write!(f, "archive profile '{profile}' backoff")
            }
            (ArchiveProfileCappedByFloor, Some(profile)) => {
                write!(f, "archive profile '{profile}' capped by wakeup floor")
            }
            (NightlyDue, _) => write!(f, "nightly maintenance due"),
            (NightlyBackoff, _) => write!(f, "nightly maintenance backoff"),
            (NightlyCappedByFloor, _) => write!(f, "nightly maintenance capped by wakeup floor"),
            (NothingDue, _) => write!(f, "nothing due anywhere"),
            (OnlineEventPending, Some(hosts)) => {
                write!(f, "waiting for online event from: {hosts}")
            }
            (HostListUnavailable, _) => {
                write!(f, "host list unavailable, retrying at wakeup floor")
            }
            // Defensive: a *Due/*Backoff/*CappedByFloor category without its subject would be
            // a construction bug elsewhere in this module, not a state this should hide.
            (category, None) => write!(f, "{category:?} (missing subject)"),
        }
    }
}

/// The scanner loop's last completed iteration and what it decided to do next.
#[derive(Debug, Clone)]
pub struct ScannerStatus {
    pub last_execution: DateTime<Local>,
    pub next_wakeup: DateTime<Local>,
    pub next_wakeup_reason: ScannerWakeupReason,
}

/// Persists the scanner loop's just-completed iteration. Best-effort: a failure here only
/// means `queueStats` temporarily reports unknown status, never affects actual scheduling.
pub async fn set_scanner_status(
    redis_client: &redis::Client,
    last_execution: DateTime<Local>,
    next_wakeup: DateTime<Local>,
    next_wakeup_reason: &ScannerWakeupReason,
) {
    let Ok(mut con) = redis_client.get_multiplexed_async_connection().await else {
        warn!("Failed to get Redis connection to persist scanner status");
        return;
    };
    let mut fields = vec![
        ("last_execution", last_execution.timestamp().to_string()),
        ("next_wakeup", next_wakeup.timestamp().to_string()),
        (
            "next_wakeup_reason_category",
            next_wakeup_reason.category.as_str().to_string(),
        ),
    ];
    if let Some(subject) = &next_wakeup_reason.subject {
        fields.push(("next_wakeup_reason_subject", subject.clone()));
    }
    if let Err(e) = con
        .hset_multiple::<_, _, _, ()>(SCANNER_STATUS_KEY, &fields)
        .await
    {
        warn!("Failed to persist scanner status: {e}");
        return;
    }
    // A category without a subject this time (e.g. a host category losing to a nightly one
    // across iterations) must not leave the previous iteration's stale subject behind.
    if next_wakeup_reason.subject.is_none() {
        if let Err(e) = con
            .hdel::<_, _, ()>(SCANNER_STATUS_KEY, "next_wakeup_reason_subject")
            .await
        {
            warn!("Failed to clear stale scanner status subject: {e}");
        }
    }
    if let Err(e) = con
        .expire::<_, ()>(
            SCANNER_STATUS_KEY,
            status_ttl_secs(last_execution, next_wakeup),
        )
        .await
    {
        warn!("Failed to set TTL on scanner status: {e}");
    }
}

/// Reads the scanner loop's last persisted status, if any (absent before the scheduler's
/// first iteration, or once the TTL above has lapsed with no scheduler running to refresh it).
pub async fn get_scanner_status(redis_client: &redis::Client) -> Option<ScannerStatus> {
    let mut con = match redis_client.get_multiplexed_async_connection().await {
        Ok(con) => con,
        Err(e) => {
            warn!("Failed to get Redis connection to read scanner status: {e}");
            return None;
        }
    };
    let fields: HashMap<String, String> = match con.hgetall(SCANNER_STATUS_KEY).await {
        Ok(fields) => fields,
        Err(e) => {
            warn!("Failed to read scanner status: {e}");
            return None;
        }
    };
    if fields.is_empty() {
        return None;
    }

    let last_execution = fields
        .get("last_execution")
        .and_then(|ts| ts.parse::<i64>().ok())
        .and_then(|ts| Local.timestamp_opt(ts, 0).single())?;
    let next_wakeup = fields
        .get("next_wakeup")
        .and_then(|ts| ts.parse::<i64>().ok())
        .and_then(|ts| Local.timestamp_opt(ts, 0).single())?;
    let category =
        ScannerWakeupReasonCategory::from_str(fields.get("next_wakeup_reason_category")?)?;
    let subject = fields.get("next_wakeup_reason_subject").cloned();

    Some(ScannerStatus {
        last_execution,
        next_wakeup,
        next_wakeup_reason: ScannerWakeupReason { category, subject },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn status_ttl_grows_with_a_far_out_next_wakeup() {
        let last_execution = Local::now();
        let next_wakeup = last_execution + Duration::days(5);
        let ttl = status_ttl_secs(last_execution, next_wakeup);
        assert_eq!(
            ttl,
            Duration::days(5).num_seconds() + SCANNER_STATUS_TTL_GRACE_SECS
        );
    }

    #[test]
    fn status_ttl_floors_when_next_wakeup_is_effectively_now() {
        let now = Local::now();
        assert_eq!(status_ttl_secs(now, now), SCANNER_STATUS_TTL_FLOOR_SECS);
    }

    #[test]
    fn status_ttl_floors_instead_of_going_negative_when_next_wakeup_precedes_last_execution() {
        let last_execution = Local::now();
        // Shouldn't happen in practice (next_wakeup is always computed after last_execution),
        // but a negative span must still clamp to the floor, not produce a negative/zero TTL
        // that would make the Redis key expire (or fail to `EXPIRE`) immediately.
        let next_wakeup = last_execution - Duration::hours(1);
        assert_eq!(
            status_ttl_secs(last_execution, next_wakeup),
            SCANNER_STATUS_TTL_FLOOR_SECS
        );
    }
}

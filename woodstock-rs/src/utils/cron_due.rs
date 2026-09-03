//! Generic "when is this cron-scheduled recurring task next due" computation.
//!
//! Shared by anything driven by a 7-part cron expression plus a persisted
//! "last run" timestamp: archive profiles (`archiving::is_profile_due`) and
//! the scheduler's nightly maintenance trigger.

use std::str::FromStr;

use chrono::{DateTime, Local};
use eyre::Result;

/// Returns the timestamp at which a task scheduled by `cron_expr` is next
/// due, given when it last ran (`None` if it never ran).
///
/// `Ok(None)` means the expression has no future fire at all (an exotic or
/// exhausted one-shot expression) — callers should treat that as "never
/// due again" and skip the task. Otherwise the returned timestamp may be in
/// the past or exactly `now` (the task is overdue/due right now — this also
/// naturally catches up a task that missed its fire time while the process
/// was down) or in the future (not due yet — usable directly as a sleep
/// target).
///
/// # Errors
/// Returns an error if `cron_expr` is not a valid cron expression.
pub fn next_due_at(
    cron_expr: &str,
    last_run: Option<DateTime<Local>>,
    now: DateTime<Local>,
) -> Result<Option<DateTime<Local>>> {
    let schedule = cron::Schedule::from_str(cron_expr)
        .map_err(|e| eyre::eyre!("Invalid cron expression '{cron_expr}': {e}"))?;

    let due_at = match last_run {
        None => Some(now),
        Some(last_run) => schedule.after(&last_run).next(),
    };

    Ok(due_at)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(y: i32, mo: u32, d: u32, h: u32, mi: u32, s: u32) -> DateTime<Local> {
        Local.with_ymd_and_hms(y, mo, d, h, mi, s).unwrap()
    }

    #[test]
    fn never_run_is_due_now() {
        let now = at(2026, 3, 7, 15, 0, 0);
        assert_eq!(
            next_due_at("0 0 3 * * SAT *", None, now).unwrap(),
            Some(now)
        );
    }

    #[test]
    fn overdue_catches_up_to_the_missed_fire_time() {
        // Every day at 03:00; process was down from before the fire until well after it.
        let cron = "0 0 3 * * * *";
        let last_run = at(2026, 3, 5, 3, 0, 0);
        let now = at(2026, 3, 6, 12, 0, 0); // well after the 2026-03-06 03:00 fire

        let due_at = next_due_at(cron, Some(last_run), now).unwrap();
        assert_eq!(due_at, Some(at(2026, 3, 6, 3, 0, 0)));
        assert!(due_at.unwrap() <= now, "overdue task must be due now");
    }

    #[test]
    fn not_yet_due_returns_future_fire_time() {
        let cron = "0 0 3 * * * *";
        let last_run = at(2026, 3, 6, 3, 0, 0);
        let now = at(2026, 3, 6, 12, 0, 0);

        let due_at = next_due_at(cron, Some(last_run), now).unwrap();
        assert_eq!(due_at, Some(at(2026, 3, 7, 3, 0, 0)));
        assert!(
            due_at.unwrap() > now,
            "not-yet-due task's fire time is in the future"
        );
    }

    #[test]
    fn invalid_cron_expression_errors() {
        assert!(next_due_at("not a cron", None, Local::now()).is_err());
    }
}

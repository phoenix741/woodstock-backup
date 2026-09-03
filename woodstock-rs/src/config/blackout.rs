use chrono::{DateTime, Datelike, Duration, Local, LocalResult, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use tracing::warn;

/// A recurring time-of-day window, on a set of week days, during which backups should
/// not be started unless [`crate::server::job::JobUtility::is_in_blackout_now`]'s override
/// condition kicks in.
///
/// `start`/`end` are `"HH:MM"` in local time. `end` may be earlier than `start` to express
/// a window crossing midnight (e.g. `22:00` -> `06:00`): the window is anchored to `days`
/// by its `start`, and spills into the following day.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlackoutWindow {
    /// Days of week this window applies to, `0` = Sunday .. `6` = Saturday
    /// (matches `chrono::Weekday::num_days_from_sunday`).
    pub days: Vec<u8>,
    pub start: String,
    pub end: String,
}

impl BlackoutWindow {
    fn parse_bound(s: &str) -> Option<NaiveTime> {
        NaiveTime::parse_from_str(s, "%H:%M").ok()
    }

    fn contains_day(&self, weekday_from_sunday: u8) -> bool {
        self.days.contains(&weekday_from_sunday)
    }
}

/// Resolves a naive `(date, time)` to a local `DateTime`, tolerating DST transitions:
/// an ambiguous local time (fall-back overlap) resolves to its *latest* interpretation,
/// so the blackout window is never shortened by DST — a nonexistent local time
/// (spring-forward gap) has no valid resolution and returns `None`.
fn resolve_local(date: NaiveDate, time: NaiveTime) -> Option<DateTime<Local>> {
    match date.and_time(time).and_local_timezone(Local) {
        LocalResult::Single(dt) => Some(dt),
        LocalResult::Ambiguous(_, latest) => Some(latest),
        LocalResult::None => None,
    }
}

/// Returns `Some(end)` if `now` falls inside one of the given windows (the latest such end,
/// if several windows overlap), `None` if `now` is not in any blackout window.
#[must_use]
pub fn blackout_status_at(
    now: DateTime<Local>,
    windows: &[BlackoutWindow],
) -> Option<DateTime<Local>> {
    let mut latest_end: Option<DateTime<Local>> = None;
    let today = now.date_naive();
    let now_time = now.time();
    let today_weekday = today.weekday().num_days_from_sunday() as u8;

    for window in windows {
        let (Some(start), Some(end)) = (
            BlackoutWindow::parse_bound(&window.start),
            BlackoutWindow::parse_bound(&window.end),
        ) else {
            warn!(
                "Ignoring blackout window with unparseable start/end (expected \"HH:MM\"): start={:?}, end={:?}",
                window.start, window.end
            );
            continue;
        };

        if start <= end {
            // Same-day window: e.g. 08:00 -> 18:00.
            if window.contains_day(today_weekday) && now_time >= start && now_time <= end {
                match resolve_local(today, end) {
                    Some(end_dt) => {
                        latest_end = Some(latest_end.map_or(end_dt, |cur| cur.max(end_dt)));
                    }
                    None => warn!(
                        "Blackout window end {end} on {today} falls in a DST spring-forward gap, skipping this occurrence"
                    ),
                }
            }
            continue;
        }

        // Overnight window: e.g. 22:00 -> 06:00, anchored on `days` by its start day.
        if window.contains_day(today_weekday) && now_time >= start {
            // Still in the first half, before midnight.
            let end_date = today + Duration::days(1);
            match resolve_local(end_date, end) {
                Some(end_dt) => {
                    latest_end = Some(latest_end.map_or(end_dt, |cur| cur.max(end_dt)));
                }
                None => warn!(
                    "Blackout window end {end} on {end_date} falls in a DST spring-forward gap, skipping this occurrence"
                ),
            }
            continue;
        }

        let yesterday = today - Duration::days(1);
        let yesterday_weekday = yesterday.weekday().num_days_from_sunday() as u8;
        if window.contains_day(yesterday_weekday) && now_time < end {
            // Spilled over from yesterday's window, still before its end this morning.
            match resolve_local(today, end) {
                Some(end_dt) => {
                    latest_end = Some(latest_end.map_or(end_dt, |cur| cur.max(end_dt)));
                }
                None => warn!(
                    "Blackout window end {end} on {today} falls in a DST spring-forward gap, skipping this occurrence"
                ),
            }
        }
    }

    latest_end
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(year: i32, month: u32, day: u32, hour: u32, min: u32) -> DateTime<Local> {
        Local
            .with_ymd_and_hms(year, month, day, hour, min, 0)
            .single()
            .unwrap()
    }

    #[test]
    fn no_windows_never_blackout() {
        assert_eq!(blackout_status_at(at(2026, 8, 31, 10, 0), &[]), None);
    }

    #[test]
    fn same_day_window_inside() {
        // Monday 2026-08-31
        let windows = vec![BlackoutWindow {
            days: vec![1], // Monday
            start: "08:00".into(),
            end: "18:00".into(),
        }];
        let status = blackout_status_at(at(2026, 8, 31, 12, 0), &windows);
        assert_eq!(status, Some(at(2026, 8, 31, 18, 0)));
    }

    #[test]
    fn same_day_window_outside() {
        let windows = vec![BlackoutWindow {
            days: vec![1],
            start: "08:00".into(),
            end: "18:00".into(),
        }];
        assert_eq!(blackout_status_at(at(2026, 8, 31, 19, 0), &windows), None);
        // Wrong day (Tuesday)
        assert_eq!(blackout_status_at(at(2026, 9, 1, 12, 0), &windows), None);
    }

    #[test]
    fn overnight_window_before_midnight() {
        // Monday 22:00 -> Tuesday 06:00, anchored on Monday.
        let windows = vec![BlackoutWindow {
            days: vec![1],
            start: "22:00".into(),
            end: "06:00".into(),
        }];
        let status = blackout_status_at(at(2026, 8, 31, 23, 0), &windows);
        assert_eq!(status, Some(at(2026, 9, 1, 6, 0)));
    }

    #[test]
    fn overnight_window_after_midnight() {
        let windows = vec![BlackoutWindow {
            days: vec![1], // Monday
            start: "22:00".into(),
            end: "06:00".into(),
        }];
        // Tuesday 03:00 is still inside the window started Monday night.
        let status = blackout_status_at(at(2026, 9, 1, 3, 0), &windows);
        assert_eq!(status, Some(at(2026, 9, 1, 6, 0)));
    }

    #[test]
    fn overnight_window_after_end() {
        let windows = vec![BlackoutWindow {
            days: vec![1],
            start: "22:00".into(),
            end: "06:00".into(),
        }];
        assert_eq!(blackout_status_at(at(2026, 9, 1, 7, 0), &windows), None);
    }

    #[test]
    fn multiple_windows_take_latest_end() {
        let windows = vec![
            BlackoutWindow {
                days: vec![1],
                start: "08:00".into(),
                end: "12:00".into(),
            },
            BlackoutWindow {
                days: vec![1],
                start: "10:00".into(),
                end: "20:00".into(),
            },
        ];
        let status = blackout_status_at(at(2026, 8, 31, 11, 0), &windows);
        assert_eq!(status, Some(at(2026, 8, 31, 20, 0)));
    }
}

//! Retention policy algorithm for backups.
//!
//! Classifies backups into sliding windows (Yearly, Monthly, Weekly,
//! Daily, Hourly) and identifies which backups should be deleted according to
//! the configured [`ScheduledBackupToKeep`] policy.

use std::collections::HashMap;

use chrono::{DateTime, Datelike, Duration, Local, Months, Timelike};
use uuid::Uuid;

use crate::config::{Backup, BackupStatus, ScheduledBackupToKeep};

/// Classification of a backup according to the retention policy.
///
/// The finest (most granular) window wins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RetentionCategory {
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Surplus,
    LastBackup,
}

fn hourly_key(dt: &DateTime<Local>) -> u64 {
    ((dt.year() as u64) << 32)
        | ((dt.month() as u64) << 24)
        | ((dt.day() as u64) << 16)
        | ((dt.hour() as u64) << 8)
}

fn daily_key(dt: &DateTime<Local>) -> u64 {
    ((dt.year() as u64) << 32) | ((dt.month() as u64) << 24) | ((dt.day() as u64) << 16)
}

fn weekly_key(dt: &DateTime<Local>) -> u64 {
    ((dt.iso_week().year() as u64) << 32) | ((dt.iso_week().week() as u64) << 16)
}

fn monthly_key(dt: &DateTime<Local>) -> u64 {
    ((dt.year() as u64) << 32) | ((dt.month() as u64) << 24)
}

fn yearly_key(dt: &DateTime<Local>) -> u64 {
    dt.year() as u64
}

fn insert_newest<'a>(map: &mut HashMap<u64, &'a Backup>, key: u64, b: &'a Backup) {
    let current = map.get(&key);
    if current.map_or(true, |best| b.start_date > best.start_date) {
        map.insert(key, b);
    }
}

pub fn classify_backups(
    backups: &[Backup],
    policy: &ScheduledBackupToKeep,
    _now: DateTime<Local>,
) -> HashMap<Uuid, RetentionCategory> {
    let mut categories: HashMap<Uuid, RetentionCategory> = HashMap::new();

    // Seed every terminal backup as Surplus
    for backup in backups {
        match &backup.status {
            BackupStatus::Completed
            | BackupStatus::Failed(_)
            | BackupStatus::Aborted
            | BackupStatus::Cancelled => {
                categories.insert(backup.id, RetentionCategory::Surplus);
            }
            _ => {}
        }
    }

    let mut completed: Vec<&Backup> = backups
        .iter()
        .filter(|b| matches!(b.status, BackupStatus::Completed))
        .collect();
    completed.sort_by(|a, b| b.start_date.cmp(&a.start_date));

    if !completed.is_empty() {
        let t0 = completed.first().unwrap().start_date;

        let b_hourly = t0 - Duration::hours(policy.hourly.unwrap_or(0) as i64);
        let b_daily = std::cmp::min(
            b_hourly,
            t0 - Duration::days(policy.daily.unwrap_or(0) as i64),
        );
        let b_weekly = std::cmp::min(
            b_daily,
            t0 - Duration::days((policy.weekly.unwrap_or(0) * 7) as i64),
        );
        let b_monthly = std::cmp::min(
            b_weekly,
            t0.checked_sub_months(Months::new(policy.monthly.unwrap_or(0) as u32))
                .unwrap_or_else(|| t0 - Duration::days(365)),
        );

        let mut hourly_reps: HashMap<u64, &Backup> = HashMap::new();
        let mut daily_reps: HashMap<u64, &Backup> = HashMap::new();
        let mut weekly_reps: HashMap<u64, &Backup> = HashMap::new();
        let mut monthly_reps: HashMap<u64, &Backup> = HashMap::new();
        let mut yearly_reps: HashMap<u64, &Backup> = HashMap::new();

        for b in &completed {
            let sd = b.start_date;

            // Notice we check >= boundaries. A backup exactly ON the boundary goes to the finer tier.
            if policy.hourly.unwrap_or(0) > 0 && sd >= b_hourly {
                insert_newest(&mut hourly_reps, hourly_key(&sd), b);
            } else if policy.daily.unwrap_or(0) > 0 && sd >= b_daily {
                insert_newest(&mut daily_reps, daily_key(&sd), b);
            } else if policy.weekly.unwrap_or(0) > 0 && sd >= b_weekly {
                insert_newest(&mut weekly_reps, weekly_key(&sd), b);
            } else if policy.monthly.unwrap_or(0) > 0 && sd >= b_monthly {
                insert_newest(&mut monthly_reps, monthly_key(&sd), b);
            } else if policy.yearly.unwrap_or(0) > 0 {
                insert_newest(&mut yearly_reps, yearly_key(&sd), b);
            }
        }

        if let Some(limit) = policy.yearly_limit {
            if limit > 0 && yearly_reps.len() > limit {
                let mut keys: Vec<u64> = yearly_reps.keys().copied().collect();
                keys.sort_unstable_by(|a, b| b.cmp(a));
                keys.truncate(limit);
                yearly_reps.retain(|k, _| keys.contains(k));
            }
        }

        for (_, b) in hourly_reps {
            categories.insert(b.id, RetentionCategory::Hourly);
        }
        for (_, b) in daily_reps {
            categories.insert(b.id, RetentionCategory::Daily);
        }
        for (_, b) in weekly_reps {
            categories.insert(b.id, RetentionCategory::Weekly);
        }
        for (_, b) in monthly_reps {
            categories.insert(b.id, RetentionCategory::Monthly);
        }
        for (_, b) in yearly_reps {
            categories.insert(b.id, RetentionCategory::Yearly);
        }
    }

    // Protect the most recent terminal backup regardless of its status —
    // not just `Completed` ones. `get_last_backup` (picks the highest
    // `number`, any status) feeds `previous_id` for the next incremental
    // backup, so a `Failed`/`Aborted`/`Cancelled` backup can be the base the
    // next run clones its manifest from (see the cancel-resume work: a
    // cancelled backup's partial data is a valid incremental base, not
    // garbage). Without this, such a backup would stay `Surplus` from the
    // seeding loop above and `get_backups_to_delete` would delete it out
    // from under the next backup's `previous_id`.
    let last_terminal_id = categories
        .keys()
        .filter_map(|id| backups.iter().find(|b| b.id == *id))
        .max_by_key(|b| b.start_date)
        .map(|b| b.id);
    if let Some(last_id) = last_terminal_id {
        if categories.get(&last_id) == Some(&RetentionCategory::Surplus) {
            categories.insert(last_id, RetentionCategory::LastBackup);
        }
    }

    categories
}

pub fn get_backups_to_delete(
    backups: &[Backup],
    policy: &ScheduledBackupToKeep,
    now: DateTime<Local>,
) -> Vec<Uuid> {
    let categories = classify_backups(backups, policy, now);
    backups
        .iter()
        .filter(|b| categories.get(&b.id) == Some(&RetentionCategory::Surplus))
        .map(|b| b.id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_backup(id: Uuid, status: BackupStatus, start_date: DateTime<Local>) -> Backup {
        Backup {
            id,
            number: 0,
            status,
            start_date,
            end_date: Some(start_date + Duration::minutes(30)),
            error_count: 0,
            error_message: None,
            file_count: 0,
            new_file_count: 0,
            removed_file_count: 0,
            modified_file_count: 0,
            existing_file_count: 0,
            file_size: 0,
            new_file_size: 0,
            modified_file_size: 0,
            existing_file_size: 0,
            compressed_file_size: 0,
            new_compressed_file_size: 0,
            modified_compressed_file_size: 0,
            existing_compressed_file_size: 0,
            speed: 0.0,
            agent_version: None,
        }
    }

    fn now_fixed() -> DateTime<Local> {
        Local.with_ymd_and_hms(2026, 3, 7, 15, 0, 0).unwrap()
    }

    fn full_policy() -> ScheduledBackupToKeep {
        ScheduledBackupToKeep {
            hourly: Some(24),
            daily: Some(7),
            weekly: Some(4),
            monthly: Some(12),
            yearly: Some(1),
            yearly_limit: None,
        }
    }

    #[test]
    fn test_empty_backups() {
        let result = classify_backups(&[], &full_policy(), now_fixed());
        assert!(result.is_empty());
    }

    #[test]
    fn test_sliding_window_logic() {
        let t0 = now_fixed(); // 2026-03-07 15:00:00

        // Hourly: >= t0 - 24h
        let id_t0 = Uuid::now_v7();
        let b_t0 = make_backup(id_t0, BackupStatus::Completed, t0);

        let id_h1 = Uuid::now_v7();
        // 2 hours ago -> same day -> hourly bucket
        let b_h1 = make_backup(id_h1, BackupStatus::Completed, t0 - Duration::hours(2));

        // Daily: >= t0 - 7d
        let id_daily0 = Uuid::now_v7();
        // 2 days ago -> daily bucket
        let b_daily0 = make_backup(id_daily0, BackupStatus::Completed, t0 - Duration::days(2));

        // Weekly: >= t0 - 28d
        let id_weekly0 = Uuid::now_v7();
        // 14 days ago -> weekly bucket
        let b_weekly0 = make_backup(id_weekly0, BackupStatus::Completed, t0 - Duration::days(14));

        // Monthly: >= t0 - 12m (approx 365d)
        let id_monthly0 = Uuid::now_v7();
        // 60 days ago -> monthly bucket
        let b_monthly0 = make_backup(
            id_monthly0,
            BackupStatus::Completed,
            t0 - Duration::days(60),
        );

        // Yearly: < t0 - 12m
        let id_yearly0 = Uuid::now_v7();
        // 400 days ago -> yearly bucket
        let b_yearly0 = make_backup(
            id_yearly0,
            BackupStatus::Completed,
            t0 - Duration::days(400),
        );

        let backups = vec![b_t0, b_h1, b_daily0, b_weekly0, b_monthly0, b_yearly0];
        let result = classify_backups(&backups, &full_policy(), t0);

        assert_eq!(result[&id_t0], RetentionCategory::Hourly);
        assert_eq!(result[&id_h1], RetentionCategory::Hourly);
        assert_eq!(result[&id_daily0], RetentionCategory::Daily);
        assert_eq!(result[&id_weekly0], RetentionCategory::Weekly);
        assert_eq!(result[&id_monthly0], RetentionCategory::Monthly);
        assert_eq!(result[&id_yearly0], RetentionCategory::Yearly);
    }

    #[test]
    fn test_calendar_monthly_bucket() {
        let t0 = now_fixed(); // 2026-03-07

        let mut backups = vec![make_backup(Uuid::now_v7(), BackupStatus::Completed, t0)];

        // Let's create two backups in the Monthly zone (older than 28 days but newer than 12 months)
        // They will be positioned in January and February exactly 29 days apart.
        // E.g., Feb 14 and Jan 16.
        let b_feb = make_backup(
            Uuid::now_v7(),
            BackupStatus::Completed,
            Local.with_ymd_and_hms(2026, 2, 5, 12, 0, 0).unwrap(),
        );
        let b_jan = make_backup(
            Uuid::now_v7(),
            BackupStatus::Completed,
            Local.with_ymd_and_hms(2026, 1, 16, 12, 0, 0).unwrap(),
        );

        backups.push(b_feb.clone());
        backups.push(b_jan.clone());

        let result = classify_backups(&backups, &full_policy(), t0);

        // Since they belong to different calendar months, they are both kept as Monthly representatives.
        assert_eq!(result[&b_feb.id], RetentionCategory::Monthly);
        assert_eq!(result[&b_jan.id], RetentionCategory::Monthly);
    }

    #[test]
    fn test_yearly_limit() {
        let t0 = now_fixed();

        let b_t0 = make_backup(Uuid::now_v7(), BackupStatus::Completed, t0);
        let b_old1 = make_backup(
            Uuid::now_v7(),
            BackupStatus::Completed,
            t0 - Duration::days(400),
        );
        let b_old2 = make_backup(
            Uuid::now_v7(),
            BackupStatus::Completed,
            t0 - Duration::days(800),
        );
        let b_old3 = make_backup(
            Uuid::now_v7(),
            BackupStatus::Completed,
            t0 - Duration::days(1200),
        );

        let backups = vec![b_t0.clone(), b_old1.clone(), b_old2.clone(), b_old3.clone()];
        let mut policy = full_policy();
        policy.yearly_limit = Some(2);

        let result = classify_backups(&backups, &policy, t0);

        assert_eq!(result[&b_t0.id], RetentionCategory::Hourly);
        assert_eq!(result[&b_old1.id], RetentionCategory::Yearly);
        assert_eq!(result[&b_old2.id], RetentionCategory::Yearly);
        assert_eq!(result[&b_old3.id], RetentionCategory::Surplus);
    }

    /// A `Cancelled` (or `Failed`/`Aborted`) backup can be the most recent
    /// terminal backup, and thus the incremental base the next backup's
    /// `previous_id` clones its manifest from. Retention must never let it
    /// be deleted as `Surplus` while it holds that role — see the doc
    /// comment on the `last_terminal_id` protection in `classify_backups`.
    #[test]
    fn test_newest_cancelled_backup_is_protected_as_last_backup() {
        let t0 = now_fixed();

        let b_completed = make_backup(
            Uuid::now_v7(),
            BackupStatus::Completed,
            t0 - Duration::hours(2),
        );
        let b_cancelled = make_backup(Uuid::now_v7(), BackupStatus::Cancelled, t0);

        let backups = vec![b_completed.clone(), b_cancelled.clone()];
        let result = classify_backups(&backups, &full_policy(), t0);

        assert_eq!(
            result[&b_cancelled.id],
            RetentionCategory::LastBackup,
            "the newest terminal backup must be protected even when Cancelled"
        );
        assert_ne!(
            result[&b_cancelled.id],
            RetentionCategory::Surplus,
            "a Cancelled backup used as the next incremental's base must not be deletable"
        );

        let to_delete = get_backups_to_delete(&backups, &full_policy(), t0);
        assert!(
            !to_delete.contains(&b_cancelled.id),
            "get_backups_to_delete must not select the protected Cancelled backup"
        );
    }
}

//! # Archiving Module
//!
//! Implements the periodic archiving feature: exporting the latest completed
//! backup of a host out of the content-addressable pool onto external media,
//! either as a portable tar-family archive or (in the future) as an
//! incremental directory mirror.
//!
//! See [`tar_writer`] for the tar/tar.gz/tar.xz writer, and [`dir_sync`] for
//! the incremental directory-mirror writer.

pub mod dir_sync;
pub mod fs_materialize;
pub mod state;
pub mod tar_writer;

pub use state::{ArchiveHostExecutionState, ArchiveHostState, ArchiveState};
pub use tar_writer::ArchiveProgressCounters;

use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};
use eyre::Result;
use serde::{Deserialize, Serialize};
use tokio::fs::read_to_string;

use crate::utils::cron_due::next_due_at;

/// Number of parallel pool-reading/decompression workers used by both
/// [`tar_writer`]'s reader side and [`dir_sync`]'s materialize side. Shared
/// here (rather than duplicated per format) since both hit the exact same
/// bottleneck: pool-chunk decompression (zstd) is CPU-bound and, per file,
/// single-threaded — with a single reader/worker this pegs one core while
/// the rest sit idle, well below what the underlying disks can sustain (see
/// `docs/developer_guide/ARCHIVING.md`'s Performance section for the
/// `iostat`/`mpstat` investigation this was introduced for).
///
/// Static, env-configurable default rather than
/// `std::thread::available_parallelism()` (unused anywhere else in this
/// codebase): `job_worker` already runs multiple job kinds concurrently in
/// the same process (`ARCHIVE_CONCURRENCY`, `BACKUP_CONCURRENCY`,
/// `MAINTENANCE_CONCURRENCY` — see `server-rs/src/jobs/config.rs`), so the
/// worst case is several of these worker sets running at once; a
/// conservative fixed default avoids one archive job claiming every core.
#[must_use]
pub(crate) fn archive_reader_worker_count() -> usize {
    parse_archive_reader_worker_count(
        std::env::var("WOODSTOCK_ARCHIVE_READER_WORKERS")
            .ok()
            .as_deref(),
    )
}

/// Parsing logic split out from [`archive_reader_worker_count`] so it's
/// testable without mutating process-global environment state
/// (`std::env::set_var` is `unsafe` and shared across concurrently-running
/// tests in this binary).
fn parse_archive_reader_worker_count(raw: Option<&str>) -> usize {
    raw.and_then(|v| v.parse().ok()).unwrap_or(4).max(1)
}

/// Tracks the last time an archive profile was triggered, so the scheduler
/// can decide whether it is due without re-deriving state from Redis job
/// history. Stored at `<jobs_path>/archiving/<profile_name>.yml`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveRunStatus {
    pub last_run: Option<DateTime<Local>>,
}

impl ArchiveRunStatus {
    fn status_path(jobs_path: &Path, profile_name: &str) -> PathBuf {
        jobs_path
            .join("archiving")
            .join(format!("{profile_name}.yml"))
    }

    /// Loads the run status for `profile_name`, defaulting to "never run" if
    /// no status file exists yet.
    ///
    /// # Errors
    /// Returns an error if the status file exists but cannot be parsed.
    pub async fn load(jobs_path: &Path, profile_name: &str) -> Result<Self> {
        let path = Self::status_path(jobs_path, profile_name);
        match read_to_string(&path).await {
            Ok(content) => Ok(serde_yaml_ng::from_str(&content)?),
            Err(_) => Ok(Self::default()),
        }
    }

    /// Persists this run status for `profile_name`.
    ///
    /// # Errors
    /// Returns an error if the status directory or file cannot be written.
    pub async fn save(&self, jobs_path: &Path, profile_name: &str) -> Result<()> {
        let path = Self::status_path(jobs_path, profile_name);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let content = serde_yaml_ng::to_string(self)?;
        tokio::fs::write(&path, content).await?;
        Ok(())
    }
}

/// Returns `true` if `profile`'s cron schedule has a fire time at or before
/// `now` that is strictly after `last_run` (or if it has never run before).
///
/// # Errors
/// Returns an error if `schedule_cron` is not a valid cron expression.
pub fn is_profile_due(
    schedule_cron: &str,
    last_run: Option<DateTime<Local>>,
    now: DateTime<Local>,
) -> Result<bool> {
    Ok(next_due_at(schedule_cron, last_run, now)?.is_some_and(|due_at| due_at <= now))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(y: i32, mo: u32, d: u32, h: u32, mi: u32, s: u32) -> DateTime<Local> {
        Local.with_ymd_and_hms(y, mo, d, h, mi, s).unwrap()
    }

    #[test]
    fn never_run_is_always_due() {
        let now = at(2026, 3, 7, 15, 0, 0);
        assert!(is_profile_due("0 0 3 * * SAT *", None, now).unwrap());
    }

    #[test]
    fn due_once_next_fire_time_has_passed() {
        // Every day at 03:00.
        let cron = "0 0 3 * * * *";
        let last_run = at(2026, 3, 5, 3, 0, 0);

        let too_early = at(2026, 3, 6, 2, 59, 0);
        assert!(!is_profile_due(cron, Some(last_run), too_early).unwrap());

        let right_on_time = at(2026, 3, 6, 3, 0, 0);
        assert!(is_profile_due(cron, Some(last_run), right_on_time).unwrap());

        let later = at(2026, 3, 10, 12, 0, 0);
        assert!(is_profile_due(cron, Some(last_run), later).unwrap());
    }

    #[test]
    fn invalid_cron_expression_errors() {
        assert!(is_profile_due("not a cron", None, Local::now()).is_err());
    }

    #[test]
    fn parse_archive_reader_worker_count_defaults_and_clamps() {
        assert_eq!(
            parse_archive_reader_worker_count(None),
            4,
            "absent value uses the default"
        );
        assert_eq!(
            parse_archive_reader_worker_count(Some("not-a-number")),
            4,
            "invalid value falls back to the default"
        );
        assert_eq!(
            parse_archive_reader_worker_count(Some("0")),
            1,
            "zero is clamped up to 1 — there must always be at least one worker"
        );
        assert_eq!(parse_archive_reader_worker_count(Some("8")), 8);
    }

    #[tokio::test]
    async fn run_status_round_trip_and_missing_file_defaults() {
        let tmp = tempfile::tempdir().unwrap();

        let missing = ArchiveRunStatus::load(tmp.path(), "no-such-profile")
            .await
            .unwrap();
        assert!(missing.last_run.is_none());

        let now = at(2026, 3, 7, 15, 0, 0);
        let status = ArchiveRunStatus {
            last_run: Some(now),
        };
        status.save(tmp.path(), "usb-weekly").await.unwrap();

        let reloaded = ArchiveRunStatus::load(tmp.path(), "usb-weekly")
            .await
            .unwrap();
        assert_eq!(reloaded.last_run, Some(now));
    }
}

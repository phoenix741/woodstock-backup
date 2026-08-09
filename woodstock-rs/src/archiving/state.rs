//! Progress state for a sequential archive run (one job, many hosts).

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::server::progression::{percent_of, speed_from};

/// Where one host stands in a sequential archive run — mirrors
/// [`crate::server::backup::save_state::ShareState`]'s role for a backup's
/// per-share progress.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveHostExecutionState {
    #[default]
    Waiting,
    InProgress,
    Success,
    Failed,
    /// Stopped by the user. The in-flight host's truncated tar (if any) is
    /// removed by the same cleanup path used for a write error; a host not
    /// yet started when the cancel arrived never runs at all.
    Cancelled,
}

/// Per-host entry of an archive run's [`ArchiveState::host_states`].
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveHostState {
    pub hostname: String,
    pub execution_state: ArchiveHostExecutionState,
    /// Bytes written for this host so far — live while `InProgress`, final
    /// once `Success`/`Failed`, `0` while still `Waiting`.
    pub progress_current: u64,
    /// Bytes expected for this host, derived the same way as
    /// [`ArchiveState::progress_max`] but scoped to just this host.
    pub progress_max: u64,
    /// Entries written for this host so far — live while `InProgress`, final
    /// once `Success`/`Failed`, `0` while still `Waiting`.
    #[serde(default)]
    pub file_count: usize,
    /// Final size in bytes of this host's produced archive — `None` until
    /// the host reaches `Success`/`Failed`. For tar-family formats this is
    /// the archive file's size on disk (post-compression), which is only
    /// known once it has been fully flushed; for `Dir` it equals
    /// `progress_current` (bytes synced, no separate archive file).
    #[serde(default)]
    pub archive_size: Option<u64>,
}

impl ArchiveHostState {
    /// Percentage of this host's `progress_max` bytes written so far, 0.0 if
    /// unknown.
    #[must_use]
    pub fn percent(&self) -> f64 {
        percent_of(self.progress_current, self.progress_max)
    }
}

/// Snapshot of a profile's archive run: how far through the sequential list
/// of hosts it is, and how many of the run's total bytes have been written
/// so far — both in aggregate (`progress_current`/`progress_max`) and broken
/// out per host (`host_states`).
///
/// `progress_max` is derived once, before the first host starts:
/// [`Backup::file_size`](crate::config::Backup) for tar-family formats (the
/// backup's own recorded size — cheap, already in memory), or the sum of
/// `Add`/`Modify` entry sizes from the diff journal for `dir` mode (since it
/// only writes the delta — using `file_size` there would wildly overestimate
/// and leave a near-no-op sync sitting at 0% before jumping to 100%).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveState {
    /// Host currently being archived, `None` before the first host starts.
    pub current_host: Option<String>,
    pub hosts_done: usize,
    pub hosts_total: usize,
    /// When this run started writing its first host — the denominator for
    /// [`ArchiveState::speed`]. Set once, right before the per-host loop
    /// starts (deliberately *after* sizing every host's `progress_max`, so
    /// that sizing time doesn't get counted as transfer time).
    #[serde(default = "Local::now")]
    pub start_date: DateTime<Local>,
    /// Bytes written so far, across all hosts processed in this run.
    pub progress_current: u64,
    /// Total bytes this run is expected to write, across all hosts.
    pub progress_max: u64,
    /// Entries written so far, across all hosts processed in this run.
    #[serde(default)]
    pub file_count: usize,
    /// Sum of [`ArchiveHostState::archive_size`] across hosts that have
    /// reached `Success`/`Failed` so far — hosts still `InProgress`/`Waiting`
    /// don't contribute yet, so this only reaches its final value once
    /// `hosts_done == hosts_total`.
    #[serde(default)]
    pub archive_size: u64,
    /// Hosts that failed during this run (skipped, not fatal to the job —
    /// see `handle_archive_run`). Empty while the run is still in progress
    /// and any host might yet fail.
    pub failed_hosts: Vec<String>,
    /// Set once the run is stopped by a user cancel. The job handler still
    /// returns `Ok(())` in this case (see `handle_archive_run`), so
    /// `JobStatus` alone reads as `COMPLETED` — this is the run-level flag
    /// callers (notably the frontend's archive task card) must check instead
    /// of inferring cancellation from `failed_hosts`, which a cancelled run
    /// deliberately leaves empty.
    #[serde(default)]
    pub cancelled: bool,
    /// One entry per host this run will touch, in processing order.
    pub host_states: Vec<ArchiveHostState>,
}

impl ArchiveState {
    /// Percentage of `progress_max` bytes written so far, 0.0 if unknown.
    #[must_use]
    pub fn percent(&self) -> f64 {
        percent_of(self.progress_current, self.progress_max)
    }

    /// Throughput of the run so far, in bytes per second — see
    /// [`speed_from`] for the shared formula (also used by
    /// [`crate::server::progression::BackupProgression::speed`]).
    #[must_use]
    pub fn speed(&self) -> f64 {
        speed_from(self.progress_current, self.start_date, None)
    }
}

impl Default for ArchiveState {
    fn default() -> Self {
        Self {
            current_host: None,
            hosts_done: 0,
            hosts_total: 0,
            start_date: Local::now(),
            progress_current: 0,
            progress_max: 0,
            file_count: 0,
            archive_size: 0,
            failed_hosts: Vec::new(),
            cancelled: false,
            host_states: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_is_zero_when_max_unknown() {
        let state = ArchiveState::default();
        assert_eq!(state.percent(), 0.0);
    }

    #[test]
    fn percent_computes_expected_ratio() {
        let state = ArchiveState {
            progress_current: 25,
            progress_max: 100,
            ..Default::default()
        };
        assert_eq!(state.percent(), 25.0);
    }

    #[test]
    fn percent_caps_conceptually_at_100_when_current_reaches_max() {
        let state = ArchiveState {
            progress_current: 100,
            progress_max: 100,
            ..Default::default()
        };
        assert_eq!(state.percent(), 100.0);
    }

    #[test]
    fn host_state_percent_computes_expected_ratio() {
        let host = ArchiveHostState {
            hostname: "srv-web".into(),
            execution_state: ArchiveHostExecutionState::InProgress,
            progress_current: 50,
            progress_max: 200,
            ..Default::default()
        };
        assert_eq!(host.percent(), 25.0);
    }

    #[test]
    fn host_state_percent_is_zero_when_max_unknown() {
        let host = ArchiveHostState::default();
        assert_eq!(host.percent(), 0.0);
    }

    #[test]
    fn speed_is_zero_when_elapsed_duration_is_zero() {
        let state = ArchiveState {
            start_date: Local::now(),
            progress_current: 1000,
            ..Default::default()
        };
        assert_eq!(state.speed(), 0.0);
    }

    #[test]
    fn speed_computes_bytes_per_second_since_start_date() {
        let state = ArchiveState {
            start_date: Local::now() - chrono::Duration::seconds(10),
            progress_current: 1000,
            ..Default::default()
        };
        assert_eq!(state.speed(), 100.0);
    }
}

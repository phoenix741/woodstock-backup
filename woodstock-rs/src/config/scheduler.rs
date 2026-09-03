use std::sync::Arc;

use eyre::Result;
use tokio::fs::read_to_string;
use tracing::debug;

use crate::config::model::{
    default_retry_backoff_after_success_secs, default_retry_backoff_on_refusal_secs,
    default_wakeup_floor_secs,
};
use crate::config::{ApplicationScheduler, ScheduledBackupToKeep};

use super::{Configuration, Schedule};

/// Service to load the global scheduler config (`defaultSchedule`), re-read fresh on every
/// call — same hot-reload-by-re-read pattern as `ArchivingConfig`.
pub struct Scheduler {
    config: Arc<Configuration>,
}

impl Scheduler {
    #[must_use]
    pub fn new(config: Arc<Configuration>) -> Self {
        Self { config }
    }

    pub async fn get_schedule(&self) -> Result<ApplicationScheduler> {
        let scheduler = read_to_string(&self.config.path.config_path_scheduler).await;
        let scheduler = match scheduler {
            Ok(data) => {
                let scheduler: ApplicationScheduler =
                    serde_yaml_ng::from_str::<ApplicationScheduler>(&data)?;
                scheduler
            }
            Err(_) => {
                debug!("Scheduler file missing, using fallback schedule");
                Self::fallback_schedule()
            }
        };
        Ok(scheduler)
    }

    fn fallback_schedule() -> ApplicationScheduler {
        ApplicationScheduler {
            // Applied once, globally, as a safety-net cap on the scanner's dynamic sleep —
            // not a per-host polling cadence anymore (see `compute_next_wakeup` in
            // `server-rs/src/bin/scheduler.rs`). Real due dates already drive the wakeup;
            // this only bounds how long a config change (new host, re-activated schedule,
            // shortened backupPeriod) can go unnoticed.
            wakeup_schedule: "0 0 * * * * *".into(),
            nightly_schedule: "0 0 0 * * * *".into(),
            default_schedule: Schedule {
                activated: Some(true),
                backup_period: Some(86400),
                backup_to_keep: Some(ScheduledBackupToKeep {
                    hourly: Some(24),
                    daily: Some(7),
                    weekly: Some(4),
                    monthly: Some(12),
                    yearly: Some(1),
                    yearly_limit: None,
                }),
                blackout: None,
                blackout_override_after_periods: None,
            },
            wakeup_floor_secs: default_wakeup_floor_secs(),
            retry_backoff_after_success_secs: default_retry_backoff_after_success_secs(),
            retry_backoff_on_refusal_secs: default_retry_backoff_on_refusal_secs(),
        }
    }
}

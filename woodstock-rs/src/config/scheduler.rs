use std::sync::Arc;

use eyre::Result;
use tokio::fs::read_to_string;
use tracing::debug;

use crate::config::{ApplicationScheduler, ScheduledBackupToKeep};

use super::{Configuration, Schedule};

/// Service pour charger le scheduler global (defaultSchedule) similaire à la version TS.
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
        // backup_period interprété en heures (24h) pour correspondre à u8
        ApplicationScheduler {
            wakeup_schedule: "0 */15 * * * * *".into(),
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
                }),
            },
        }
    }
}

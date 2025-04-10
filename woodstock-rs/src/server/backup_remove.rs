use eyre::Result;
use log::info;
use std::time::SystemTime;

use crate::{
    config::{Backups, Configuration, Context},
    events::create_event_backup_remove,
    pool::{Refcnt, RefcntApplySens},
    EventSource,
};

pub struct BackupRemove {
    hostname: String,
    current_backup_id: usize,

    source: EventSource,
    config: Configuration,
}

impl BackupRemove {
    #[must_use]
    pub fn new(
        hostname: &str,
        backup_number: usize,
        ctxt: &Context,
        config: &Configuration,
    ) -> Self {
        let backups = Backups::new(config);
        let destination_directory =
            backups.get_backup_destination_directory(hostname, backup_number);

        info!(
            "Initialize backup remover for {hostname}/{backup_number} in {destination_directory:?}"
        );

        BackupRemove {
            hostname: hostname.to_string(),
            current_backup_id: backup_number,
            source: ctxt.source,
            config: config.clone(),
        }
    }

    pub async fn remove_refcnt_of_host(&self) -> Result<()> {
        let backups = Backups::new(&self.config);
        let from_directory =
            backups.get_backup_destination_directory(&self.hostname, self.current_backup_id);

        let host_directory = backups.get_host_path(&self.hostname);

        let mut backup_refcnt = Refcnt::new(&from_directory);
        backup_refcnt.load_refcnt(false).await;

        Refcnt::apply_all_from(
            &host_directory,
            &backup_refcnt,
            &RefcntApplySens::Decrease,
            &SystemTime::now(),
            &self.config,
        )
        .await?;

        Ok(())
    }

    pub async fn remove_backup(&self) -> Result<()> {
        let backups = Backups::new(&self.config);
        backups
            .remove_backup(&self.hostname, self.current_backup_id)
            .await?;

        let shares = backups
            .get_backup_share_paths(&self.hostname, self.current_backup_id)
            .await;
        let shares = shares
            .iter()
            .map(std::string::String::as_str)
            .collect::<Vec<&str>>();

        create_event_backup_remove(
            &self.config.path.events_path,
            self.source,
            &self.hostname,
            self.current_backup_id,
            &shares,
        )
        .await?;

        Ok(())
    }
}

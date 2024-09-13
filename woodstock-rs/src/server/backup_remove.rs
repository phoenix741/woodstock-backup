use eyre::Result;
use log::info;
use std::time::SystemTime;

use crate::{
    config::{Backups, Context},
    events::create_event_backup_remove,
    pool::{Refcnt, RefcntApplySens},
    EventSource,
};

pub struct BackupRemove {
    hostname: String,
    current_backup_id: usize,

    source: EventSource,
    context: Context,
}

impl BackupRemove {
    pub fn new(hostname: &str, backup_number: usize, ctxt: &Context) -> Self {
        let backups = Backups::new(ctxt);
        let destination_directory =
            backups.get_backup_destination_directory(hostname, backup_number);

        info!(
            "Initialize backup client for {hostname}/{backup_number} in {destination_directory:?}"
        );

        BackupRemove {
            hostname: hostname.to_string(),
            current_backup_id: backup_number,
            source: ctxt.source,
            context: ctxt.clone(),
        }
    }

    pub async fn remove_refcnt_of_host(&self) -> Result<()> {
        let backups = Backups::new(&self.context);
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
            &self.context,
        )
        .await?;

        Ok(())
    }

    pub async fn remove_backup(&self) -> Result<()> {
        let backups = Backups::new(&self.context);
        backups
            .remove_backup(&self.hostname, self.current_backup_id)
            .await?;

        let shares = backups
            .get_backup_share_paths(&self.hostname, self.current_backup_id)
            .await;
        let shares = shares.iter().map(|s| s.as_str()).collect::<Vec<&str>>();

        create_event_backup_remove(
            &self.context.config.path.events_path,
            self.source,
            &self.hostname,
            self.current_backup_id,
            &shares,
        )
        .await?;

        Ok(())
    }
}

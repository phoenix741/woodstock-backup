mod backuppc_client;
mod backuppc_manifest;

use std::cmp::min;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::UNIX_EPOCH;

use backuppc_client::BackupPCClient;
use backuppc_pool_reader::hosts::{Hosts as BackupPCHosts, HostsTrait};
use clap::{command, Parser};
use console::Emoji;
use console::Term;
use indicatif::MultiProgress;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use log::error;
use log::info;
use woodstock::config::{Backups, Context, Hosts};
use woodstock::pool::Refcnt;
use woodstock::pool::RefcntApplySens;
use woodstock::server::backup_client::BackupClient;
use woodstock::Share;

#[derive(Debug)]
struct BackupDefinition {
    pub hostname: String,

    pub backup_number: usize,

    pub start_time: u64,

    pub size: u64,
}

fn list_woodstock_backups(ctxt: &Context) -> Vec<BackupDefinition> {
    let mut result = Vec::new();

    let hosts_config = Hosts::new(ctxt);
    let backups_config = Backups::new(ctxt);

    let hosts = hosts_config.list_hosts().unwrap_or_default();
    for host in hosts {
        let backups = backups_config.get_backups(&host);
        for backup in backups {
            result.push(BackupDefinition {
                hostname: host.clone(),
                backup_number: backup.number,
                start_time: backup.start_date,
                size: backup.file_size,
            });
        }
    }

    result
}

fn list_backuppc_backups(pool_path: &str) -> Vec<BackupDefinition> {
    let mut result = Vec::new();

    let hosts_config = BackupPCHosts::new(pool_path);

    let hosts = hosts_config.list_hosts().unwrap_or_default();
    for host in hosts {
        let backups = hosts_config.list_backups(&host).unwrap_or_default();
        for backup in backups {
            result.push(BackupDefinition {
                hostname: host.clone(),
                backup_number: backup.num as usize,
                start_time: backup.start_time,
                size: backup.size,
            });
        }
    }

    result
}

pub async fn add_refcnt_to_pool(
    ctxt: &Context,
    from_directory: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Add refcnt to pool for {}", from_directory.display());
    let mut backup_refcnt = Refcnt::new(from_directory, ctxt);
    backup_refcnt.load_refcnt(false).await;

    info!("Apply refcnt to pool");
    Refcnt::apply_all_from(
        &ctxt.config.path.pool_path,
        &backup_refcnt,
        &RefcntApplySens::Increase,
        ctxt,
    )
    .await?;

    info!("Refcnt applied to pool");

    Ok(())
}

async fn launch_backup(
    context: &Context,
    backuppc_pool: &str,
    backup: &BackupDefinition,
    backup_bar: &mut ProgressBar,
) -> Result<(), Box<dyn std::error::Error>> {
    let backups_configuration = Backups::new(context);
    let backuppc_configuration = BackupPCHosts::new(backuppc_pool);
    let backuppc_shares = backuppc_configuration
        .list_shares(&backup.hostname, u32::try_from(backup.backup_number)?)?;

    let backup_number = match backups_configuration.get_last_backup(&backup.hostname) {
        Some(backup) => backup.number + 1,
        None => 0,
    };

    let destination_directory =
        backups_configuration.get_backup_destination_directory(&backup.hostname, backup_number);

    let mut abort = false;

    let message = format!("Backuping {}", &backup.hostname);
    backup_bar.set_message(message);
    backup_bar.tick();

    let backuppc_client =
        BackupPCClient::new(backuppc_pool, &backup.hostname, backup.backup_number);

    let mut client = BackupClient::new(backuppc_client, &backup.hostname, backup_number, context);
    client.set_fake_date(UNIX_EPOCH.checked_add(Duration::from_secs(backup.start_time)));

    backup_bar.set_message("Create backup directory");
    backup_bar.tick();

    client.init_backup_directory().await?;

    backup_bar.set_message("Upload last file list");
    backup_bar.tick();

    if let Err(err) = client.upload_file_list(backuppc_shares.clone()).await {
        error!("Error uploading file list: {}", err);
        abort = true;
    }

    backup_bar.set_message("Download file list");
    backup_bar.tick();

    for share in &backuppc_shares {
        let share = Share {
            includes: Vec::new(),
            excludes: Vec::new(),
            share_path: share.clone(),
        };

        if !abort {
            if let Err(err) = client.download_file_list(&share, &|_| {}).await {
                error!("Error downloading file list: {}", err);
                abort = true;
            }
        }
    }

    backup_bar.set_message("Download chunks");
    backup_bar.tick();

    backup_bar.set_length(client.progress().await.progress_max);
    for share in &backuppc_shares {
        let message = format!("Downloading chunks for {}", &share);
        backup_bar.set_message(message);
        backup_bar.tick();

        if !abort {
            if let Err(err) = client
                .create_backup(share, &|progress| {
                    backup_bar.set_position(min(progress.progress_current, progress.progress_max));
                })
                .await
            {
                error!("Error downloading chunks: {}", err);
                abort = true;
            }
        }
    }

    backup_bar.set_message("Compact manifests");
    backup_bar.tick();

    for share in &backuppc_shares {
        client.compact(share).await?;
    }

    backup_bar.set_message("Count reference of backup");
    backup_bar.tick();

    client.count_references().await?;

    client.save_backup(!abort).await?;

    backup_bar.set_message("Add reference counting to pool");
    backup_bar.tick();
    add_refcnt_to_pool(context, &destination_directory).await?;

    Ok(())
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to the file to read
    backuppc_pool: String,

    /// The type of the file to read
    woodstock_pool: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Cli::parse();
    let term = Term::stdout();

    term.write_line(&format!(
        "[1/3] {}Import BackupPC pool {} to Woodstock pool {}",
        Emoji("➡️ ", ""),
        args.backuppc_pool,
        args.woodstock_pool
    ))?;

    let context = Context::new(PathBuf::from(args.woodstock_pool), log::Level::Info);

    let woodstock_backups = list_woodstock_backups(&context);
    let backuppc_backups = list_backuppc_backups(&args.backuppc_pool);

    // Remove from backuppc_backups the backups that are already in woodstock_backups
    let backuppc_backups = backuppc_backups
        .into_iter()
        .filter(|backup| {
            !woodstock_backups.iter().any(|woodstock_backup| {
                woodstock_backup.hostname == backup.hostname
                    && woodstock_backup.start_time == backup.start_time
            })
        })
        .collect::<Vec<_>>();

    term.write_line(&format!(
        "[2/3] {}Found {} backuppc backups",
        Emoji("💽 ", ""),
        backuppc_backups.len()
    ))?;

    let size = backuppc_backups
        .iter()
        .map(|b| b.size)
        .reduce(|acc, size| acc + size)
        .unwrap_or_default();
    let length = backuppc_backups.len();

    let multi = MultiProgress::new();

    let total_bar = multi.add(ProgressBar::new(size));
    total_bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {percent_precise:>7}% {msg:>65} ETA: {eta}",
        )
        .unwrap(),
    );
    total_bar.set_message(format!("{}/{}", 0, length));
    total_bar.tick();

    for (count, backup) in backuppc_backups.into_iter().enumerate() {
        let mut backup_bar = multi.add(ProgressBar::new(1));

        backup_bar.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {percent_precise:>7}% ({bytes_per_sec:>12}) {msg:>50} ETA: {eta}",
            )
            .unwrap()
        );

        launch_backup(&context, &args.backuppc_pool, &backup, &mut backup_bar).await?;

        backup_bar.finish();
        multi.remove(&backup_bar);

        total_bar.inc(backup.size);
        total_bar.set_message(format!("{}/{}", count + 1, length));
    }
    total_bar.finish();

    term.write_line(&format!("[3/3] {}Backups migrate", Emoji("🪄 ", "")))?;

    Ok(())
}

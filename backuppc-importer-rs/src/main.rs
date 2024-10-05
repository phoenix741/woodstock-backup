mod backuppc_client;
mod backuppc_manifest;

use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use backuppc_client::BackupPCClient;
use backuppc_pool_reader::attribute_file::Search;
use backuppc_pool_reader::hosts::{Hosts as BackupPCHosts, HostsTrait};
use backuppc_pool_reader::util::osstr_to_vec;
use backuppc_pool_reader::util::vec_to_osstr;
use backuppc_pool_reader::view::BackupPC;
use clap::{command, Parser};
use console::Emoji;
use console::Term;
use eyre::Result;
use indicatif::MultiProgress;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use log::debug;
use log::error;
use log::info;
use woodstock::config::{Backups, Context, Hosts};
use woodstock::pool::remove_refcnt_to_pool;
use woodstock::pool::Refcnt;
use woodstock::pool::RefcntApplySens;
use woodstock::server::backup_client::BackupClient;
use woodstock::server::backup_remove::BackupRemove;
use woodstock::Share;

#[derive(Debug)]
struct BackupDefinition {
    pub hostname: String,

    pub backup_number: usize,

    pub start_time: u64,

    pub size: u64,
}

async fn list_woodstock_backups(ctxt: &Context) -> Vec<BackupDefinition> {
    let mut result = Vec::new();

    let hosts_config = Hosts::new(ctxt);
    let backups_config = Backups::new(ctxt);

    let hosts = hosts_config.list_hosts().await.unwrap_or_default();
    for host in hosts {
        let backups = backups_config.get_backups(&host).await;
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
            let host = vec_to_osstr(&host);
            result.push(BackupDefinition {
                hostname: host.to_string_lossy().to_string(),
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
    date: &SystemTime,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Add refcnt to pool for {}", from_directory.display());
    let mut backup_refcnt = Refcnt::new(from_directory);
    backup_refcnt.load_refcnt(false).await;

    info!("Apply refcnt to pool");
    Refcnt::apply_all_from(
        &ctxt.config.path.pool_path,
        &backup_refcnt,
        &RefcntApplySens::Increase,
        date,
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
    let search = Search::new(backuppc_pool);
    let mut view = BackupPC::new(
        backuppc_pool,
        Box::new(backuppc_configuration),
        Box::new(search),
    );

    let hostname = osstr_to_vec(&OsString::from(&backup.hostname));
    let backuppc_shares = view.list_shares(&hostname, u32::try_from(backup.backup_number)?)?;

    let backuppc_client = BackupPCClient::new(view, &backup.hostname, backup.backup_number);

    let backup_number = match backups_configuration
        .get_last_backup(&backup.hostname)
        .await
    {
        Some(backup) => backup.number + 1,
        None => 0,
    };

    let destination_directory =
        backups_configuration.get_backup_destination_directory(&backup.hostname, backup_number);

    let mut abort = false;

    let message = format!("Backuping {}", &backup.hostname);
    backup_bar.set_message(message);
    backup_bar.tick();

    let mut client = BackupClient::new(backuppc_client, &backup.hostname, backup_number, context);
    client.set_fake_date(UNIX_EPOCH.checked_add(Duration::from_secs(backup.start_time)));

    backup_bar.set_message("Create backup directory");
    backup_bar.tick();

    let backuppc_shares: Vec<String> = backuppc_shares
        .iter()
        .map(|s| vec_to_osstr(s))
        .filter_map(|s| s.into_string().ok())
        .collect();
    let backuppc_shares_str: Vec<&str> = backuppc_shares
        .iter()
        .map(std::string::String::as_str)
        .collect();
    client.init_backup_directory(&backuppc_shares_str).await?;

    backup_bar.set_message("Synchronize file list");
    backup_bar.tick();

    for share in &backuppc_shares {
        let share = Share {
            includes: Vec::new(),
            excludes: Vec::new(),
            share_path: share.clone(),
        };

        if !abort {
            if let Err(err) = client.synchronize_file_list(&share, &|_| {}).await {
                error!("Error synchronize file list: {}", err);
                abort = true;
            }
        }
    }

    backup_bar.set_message("Download chunks");
    backup_bar.tick();

    let progress_max = client.progress().await.progress_max;
    backup_bar.set_length(progress_max);
    for share in &backuppc_shares {
        let progress_min = client.progress().await.progress_current;

        let message = format!("Downloading chunks for {}", &share);
        backup_bar.set_message(message);
        backup_bar.tick();

        if !abort {
            if let Err(err) = client
                .create_backup(share, &|progress| {
                    backup_bar.set_position(progress_min + progress.progress_current);
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

    client.save_backup(true, !abort).await?;

    backup_bar.set_message("Add reference counting to pool");
    backup_bar.tick();
    add_refcnt_to_pool(context, &destination_directory, &client.get_fake_date()).await?;

    Ok(())
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to the file to read
    backuppc_pool: String,

    /// The type of the file to read
    woodstock_pool: String,

    /// Option to set the transfert of only one
    #[clap(short, long)]
    only_one: bool,

    /// Dry run
    #[clap(short, long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install()?;
    env_logger::init();
    let args = Cli::parse();
    let term = Term::stdout();

    let context = Context::new(
        PathBuf::from(args.woodstock_pool.clone()),
        log::Level::Info,
        woodstock::EventSource::Import,
        None,
        1,
    );

    // Write version
    term.write_line(&format!(
        "BackupPC to Woodstock migration tool v{}",
        woodstock::config::Configuration::version()
    ))?;

    // Display path used to make the migration
    term.write_line("Woodstock path:")?;
    term.write_line(&format!(
        "  - Backup:      {:?}",
        context.config.path.backup_path
    ))?;
    term.write_line(&format!(
        "  - Certificate: {:?}",
        context.config.path.certificates_path
    ))?;
    term.write_line(&format!(
        "  - Config:      {:?}",
        context.config.path.config_path
    ))?;
    term.write_line(&format!(
        "  - Hosts:       {:?}",
        context.config.path.hosts_path
    ))?;
    term.write_line(&format!(
        "  - Logs:        {:?}",
        context.config.path.logs_path
    ))?;
    term.write_line(&format!(
        "  - Events:      {:?}",
        context.config.path.events_path
    ))?;
    term.write_line(&format!(
        "  - Pool:        {:?}",
        context.config.path.pool_path
    ))?;
    term.write_line(&format!(
        "  - Jobs:        {:?}",
        context.config.path.jobs_path
    ))?;

    term.write_line(&format!(
        "[1/4] {}Import BackupPC pool {} to Woodstock pool {}",
        Emoji("➡️ ", ""),
        args.backuppc_pool,
        args.woodstock_pool
    ))?;

    let mut woodstock_backups = list_woodstock_backups(&context).await;
    woodstock_backups.sort_by_key(|backup| backup.start_time);

    for woodstock in &woodstock_backups {
        debug!(
            "Woodstock backup {}/{}: {} (start at {})",
            woodstock.hostname, woodstock.backup_number, woodstock.size, woodstock.start_time
        );
    }

    let mut backuppc_backups = list_backuppc_backups(&args.backuppc_pool);
    backuppc_backups.sort_by_key(|backup| backup.start_time);

    for backuppc in &backuppc_backups {
        debug!(
            "BackupPC backup {}/{}: {} (start at {})",
            backuppc.hostname, backuppc.backup_number, backuppc.size, backuppc.start_time
        );
    }

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

    // If only one, keep the first only
    let backuppc_backups = if args.only_one {
        backuppc_backups.into_iter().take(1).collect::<Vec<_>>()
    } else {
        backuppc_backups
    };

    term.write_line(&format!(
        "[2/4] {}Found {} backuppc backups",
        Emoji("💽 ", ""),
        backuppc_backups.len()
    ))?;

    let size = backuppc_backups
        .iter()
        .map(|b| b.size)
        .reduce(|acc, size| acc + size)
        .unwrap_or_default();
    let length = backuppc_backups.len();

    for backuppc in &backuppc_backups {
        info!(
            "BackupPC backup {}/{}: {}",
            backuppc.hostname, backuppc.backup_number, backuppc.size
        );
    }

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

        if !args.dry_run {
            let result =
                launch_backup(&context, &args.backuppc_pool, &backup, &mut backup_bar).await;
            if let Err(err) = result {
                error!(
                    "Error during backup of {}/{}: {}",
                    backup.hostname, backup.backup_number, err
                );
            }
        }

        backup_bar.finish();
        multi.remove(&backup_bar);

        total_bar.inc(backup.size);
        total_bar.set_message(format!("{}/{}", count + 1, length));
    }
    total_bar.finish();

    if !args.only_one {
        // List backups in woodstock that is not in backuppc
        let mut woodstock_backups = list_woodstock_backups(&context).await;
        woodstock_backups.sort_by_key(|backup| backup.start_time);

        let mut backuppc_backups = list_backuppc_backups(&args.backuppc_pool);
        backuppc_backups.sort_by_key(|backup| backup.start_time);

        // Remove from backuppc_backups the backups that are already in woodstock_backups
        let woodstock_backups_to_remove = woodstock_backups
            .into_iter()
            .filter(|backup| {
                !backuppc_backups.iter().any(|backuppc_backup| {
                    backuppc_backup.hostname == backup.hostname
                        && backuppc_backup.start_time == backup.start_time
                })
            })
            .collect::<Vec<_>>();

        for woodstock in &woodstock_backups_to_remove {
            info!(
                "Backup to remove {}/{}: {}",
                woodstock.hostname, woodstock.backup_number, woodstock.size
            );
        }

        term.write_line(&format!(
            "[3/4] {}Remove {} old backups",
            Emoji("🗑️ ", ""),
            woodstock_backups_to_remove.len()
        ))?;

        let total_bar = ProgressBar::new(size);
        total_bar.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {percent_precise:>7}% {msg:>65} ETA: {eta}",
            )
            .unwrap(),
        );
        total_bar.set_message(format!("{}/{}", 0, length));
        total_bar.tick();

        for (count, backup) in woodstock_backups_to_remove.into_iter().enumerate() {
            if !args.dry_run {
                let remover = BackupRemove::new(&backup.hostname, backup.backup_number, &context);

                remove_refcnt_to_pool(&context, &backup.hostname, backup.backup_number).await?;

                remover.remove_refcnt_of_host().await?;

                remover.remove_backup().await?;
            }

            total_bar.inc(backup.size);
            total_bar.set_message(format!("{}/{}", count + 1, length));
        }
        total_bar.finish();
    }

    term.write_line(&format!("[4/4] {}Backups migrate", Emoji("🪄 ", "")))?;

    Ok(())
}

//! Main entry point for the `BackupPC` importer application.
//!
//! This crate provides tools and utilities to import and synchronize backups from a `BackupPC` pool into the Woodstock backup system.
//! It includes logic for listing, comparing, and transferring backup data, as well as managing backup metadata and progress reporting.

//! Module for `BackupPC` client logic and Woodstock synchronization.
pub mod backuppc_client;
/// Module for `BackupPC` manifest structures and conversions.
pub mod backuppc_manifest;

use std::ffi::OsString;
use std::sync::Arc;

use backuppc_client::BackupPCClient;
use backuppc_pool_reader::attribute_file::Search;
use backuppc_pool_reader::hosts::{Hosts as BackupPCHosts, HostsTrait};
use backuppc_pool_reader::util::osstr_to_vec;
use backuppc_pool_reader::util::vec_to_osstr;
use backuppc_pool_reader::view::BackupPC;
use chrono::DateTime;
use chrono::Local;
use clap::Parser;
use console::Emoji;
use console::Term;
use eyre::Result;
use indicatif::MultiProgress;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use tokio::sync::mpsc;
use tracing::debug;
use tracing::error;
use tracing::info;
use uuid::Uuid;
use woodstock::config::Configuration;
use woodstock::config::DEFAULT_CHANNEL_BUFFER_SIZE;
use woodstock::config::GLOBAL_CONFIGURATION;
use woodstock::config::{BackupStatus, Scheduler};
use woodstock::config::{Backups, Context, Hosts};
use woodstock::pool::PoolManager;
use woodstock::server::backup::remove::BackupRemove;
use woodstock::server::backup::save::BackupSave;
use woodstock::server::progression::BackupProgression;
use woodstock::utils::lock_redis::{ImportLockOperation, LockOperation, PoolLockRedis};
use woodstock::Share;

/// Shared state for the `BackupPC` importer application.
struct ServiceState {
    pub config: Arc<Configuration>,
    pub hosts: Arc<Hosts>,
    pub backups: Arc<Backups>,
}

impl Default for ServiceState {
    fn default() -> Self {
        let config = Arc::new(GLOBAL_CONFIGURATION.clone());
        let scheduler = Arc::new(Scheduler::new(config.clone()));
        let hosts = Arc::new(Hosts::new(config.clone(), scheduler));
        let backups = Arc::new(Backups::new(config.clone()));
        ServiceState {
            config,
            hosts,
            backups,
        }
    }
}

/// Represents a backup definition for a host in the `BackupPC` or Woodstock system.
///
/// This struct contains metadata about a specific backup, including the host name, backup number, start time, and size.
#[derive(Debug)]
struct BackupDefinition {
    /// The name of the host for which the backup is defined.
    pub hostname: String,
    /// The unique UUID identifier of the backup.
    pub backup_id: uuid::Uuid,
    /// The backup number associated with the host.
    pub backup_number: usize,
    /// The start time of the backup (Unix timestamp).
    pub start_time: DateTime<Local>,
    /// The size of the backup in bytes.
    pub size: u64,
}

fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Lists all Woodstock backups, excluding those matching the provided patterns.
///
/// # Arguments
/// * `config` - The Woodstock configuration to use for listing backups.
/// * `excludes` - A slice of string patterns to exclude from the results.
///
/// # Returns
/// A vector of `BackupDefinition` containing metadata for each backup found.
async fn list_woodstock_backups(
    state: Arc<ServiceState>,
    excludes: &[&str],
) -> Vec<BackupDefinition> {
    let mut result = Vec::new();

    let hosts = state.hosts.list_hosts().await.unwrap_or_default();
    for host in hosts {
        if excludes.iter().any(|exclude| host.contains(exclude)) {
            continue;
        }

        let backups = state.backups.get_backups(&host).await;
        for backup in backups {
            result.push(BackupDefinition {
                hostname: host.clone(),
                backup_id: backup.id,
                backup_number: backup.number,
                start_time: backup.start_date,
                size: backup.file_size,
            });
        }
    }

    result
}

/// Lists all `BackupPC` backups in the specified pool path, excluding those matching the provided patterns.
///
/// # Arguments
/// * `pool_path` - The path to the `BackupPC` pool.
/// * `excludes` - A slice of string patterns to exclude from the results.
///
/// # Returns
/// A vector of `BackupDefinition` containing metadata for each backup found in the `BackupPC` pool.
fn list_backuppc_backups(pool_path: &str, excludes: &[&str]) -> Vec<BackupDefinition> {
    let mut result = Vec::new();

    let hosts_config = BackupPCHosts::new(pool_path);

    let hosts = hosts_config.list_hosts().unwrap_or_default();
    for host in hosts {
        let hostname = vec_to_osstr(&host).to_string_lossy().to_string();
        if excludes.iter().any(|exclude| hostname.contains(exclude)) {
            continue;
        }

        let backups = hosts_config.list_backups(&host).unwrap_or_default();
        for backup in backups {
            result.push(BackupDefinition {
                hostname: hostname.clone(),
                backup_id: uuid::Uuid::nil(), // BackupPC backups have no Woodstock UUID; used for comparison only
                backup_number: backup.num as usize,
                start_time: DateTime::from_timestamp(backup.start_time as i64, 0)
                    .map(DateTime::<Local>::from)
                    .unwrap_or_else(|| Local::now()),
                size: backup.size,
            });
        }
    }

    result
}

/// Launches a backup operation from a `BackupPC` pool for a specific backup definition.
///
/// # Arguments
/// * `context` - The Woodstock context for the operation.
/// * `backuppc_pool` - The path to the `BackupPC` pool.
/// * `backup` - The backup definition to process.
/// * `backup_bar` - The progress bar to update during the operation.
///
/// # Returns
/// Returns `Ok(())` if the backup operation succeeds, or an error if it fails.
///
/// # Errors
/// Returns an error if the backup operation fails, if the `BackupPC` pool cannot be accessed, or if there are issues with the backup definition.
async fn launch_backup(
    context: &Context,
    state: Arc<ServiceState>,
    backuppc_pool: &str,
    backup: &BackupDefinition,
    backup_bar: &mut ProgressBar,
) -> Result<()> {
    let backuppc_configuration = BackupPCHosts::new(backuppc_pool);
    let search = Search::new(backuppc_pool);
    let mut view = BackupPC::new(
        backuppc_pool,
        Box::new(backuppc_configuration),
        Box::new(search),
    );

    let hostname = osstr_to_vec(&OsString::from(&backup.hostname));
    let backuppc_shares = view
        .list_shares(&hostname, u32::try_from(backup.backup_number)?)
        .map_err(|e| eyre::eyre!("{e}"))?;

    let backuppc_client = BackupPCClient::new(
        view,
        &backup.hostname,
        backup.backup_number,
        &GLOBAL_CONFIGURATION.chunk_algorithm,
    );

    let last_backup = state.backups.get_last_backup(&backup.hostname).await;
    let previous_id = last_backup.as_ref().map(|b| b.id);
    let backup_number = last_backup.map(|b| b.number + 1).unwrap_or(0);
    let backup_id = Uuid::now_v7();

    let mut abort = false;

    let message = format!("Backuping {}", &backup.hostname);
    backup_bar.set_message(message);
    backup_bar.tick();

    let mut client = BackupSave::new(
        backuppc_client,
        &backup.hostname,
        backup_id,
        backup_number,
        previous_id,
        context,
        state.config.clone(),
        state.backups.clone(),
        tokio_util::sync::CancellationToken::new(),
    );
    client.set_fake_date(Some(backup.start_time));
    client.set_agent_version(version()).await;

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
            if let Err(err) = client.synchronize_file_list(&share, None).await {
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
            // Créer un canal pour recevoir les mises à jour de progression
            let (tx, mut rx) = mpsc::channel::<BackupProgression>(DEFAULT_CHANNEL_BUFFER_SIZE);

            // Spawn une tâche pour traiter les mises à jour de progression
            let backup_bar_clone = backup_bar.clone();
            let progress_task = tokio::spawn(async move {
                while let Some(progress) = rx.recv().await {
                    backup_bar_clone.set_position(progress_min + progress.progress_current);
                }
            });

            // Appeler create_backup avec le canal de progression
            let result = client.create_backup(share, Some(tx)).await;

            // Attendre que la tâche de mise à jour de progression se termine
            let _ = progress_task.await;

            if let Err(err) = result {
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
    client.add_refcnt_to_pool().await?;

    client
        .save_backup(if abort {
            BackupStatus::Failed(woodstock::config::FailedStatus::Compact)
        } else {
            BackupStatus::Completed
        })
        .await?;

    backup_bar.set_message("Add reference counting to pool");
    backup_bar.tick();

    {
        let redis_url = state.config.redis_url();
        let _lock = PoolLockRedis::new_with_path(
            &redis_url,
            &state.config.path.pool_path,
            LockOperation::Import(ImportLockOperation::Refcnt),
        )
        .await?
        .lock_exclusive()
        .await?;
        PoolManager::new(state.config.clone())
            .apply_pending(&client.get_fake_date())
            .await?;
    }

    Ok(())
}

/// Command-line interface options for the `BackupPC` importer application.
///
/// This struct defines the available command-line arguments for configuring the import process, including pool paths, transfer options, and exclusions.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to the `BackupPC` pool to read from.
    backuppc_pool: String,

    /// The path to the Woodstock pool to read from or write to.
    woodstock_pool: String,

    /// Option to transfer only one backup.
    #[clap(short, long)]
    only_one: bool,

    /// Perform a dry run without making any changes.
    #[clap(short, long)]
    dry_run: bool,

    /// List of patterns to exclude from the import process.
    #[clap(long)]
    excludes: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    let args = Cli::parse();
    let term = Term::stdout();

    let context = Context {
        source: woodstock::EventSource::Import,
        username: None,
    };
    GLOBAL_CONFIGURATION.fix_algorithm()?;

    let state = Arc::new(ServiceState::default());

    // Write version
    term.write_line(&format!(
        "`BackupPC` to Woodstock migration tool v{}",
        version()
    ))?;

    // Display path used to make the migration
    term.write_line("Woodstock path:")?;
    term.write_line(&format!(
        "  - Backup:      {:?}",
        GLOBAL_CONFIGURATION.path.backup_path
    ))?;
    term.write_line(&format!(
        "  - Certificate: {:?}",
        GLOBAL_CONFIGURATION.path.certificates_path
    ))?;
    term.write_line(&format!(
        "  - Config:      {:?}",
        GLOBAL_CONFIGURATION.path.config_path
    ))?;
    term.write_line(&format!(
        "  - Hosts:       {:?}",
        GLOBAL_CONFIGURATION.path.hosts_path
    ))?;
    term.write_line(&format!(
        "  - Logs:        {:?}",
        GLOBAL_CONFIGURATION.path.logs_path
    ))?;
    term.write_line(&format!(
        "  - Events:      {:?}",
        GLOBAL_CONFIGURATION.path.events_path
    ))?;
    term.write_line(&format!(
        "  - Pool:        {:?}",
        GLOBAL_CONFIGURATION.path.pool_path
    ))?;
    term.write_line(&format!(
        "  - Jobs:        {:?}",
        GLOBAL_CONFIGURATION.path.jobs_path
    ))?;

    term.write_line(&format!(
        "[1/4] {}Import `BackupPC` pool {} to Woodstock pool {}",
        Emoji("➡️ ", ""),
        args.backuppc_pool,
        args.woodstock_pool
    ))?;

    let excludes: Vec<&str> = args
        .excludes
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(AsRef::as_ref)
        .collect();

    let mut woodstock_backups = list_woodstock_backups(state.clone(), &excludes).await;
    woodstock_backups.sort_by_key(|backup| backup.start_time);

    for woodstock in &woodstock_backups {
        debug!(
            "Woodstock backup {}/{}: {} (start at {})",
            woodstock.hostname, woodstock.backup_number, woodstock.size, woodstock.start_time
        );
    }

    let mut backuppc_backups = list_backuppc_backups(&args.backuppc_pool, &excludes);
    backuppc_backups.sort_by_key(|backup| backup.start_time);

    for backuppc in &backuppc_backups {
        debug!(
            "`BackupPC` backup {}/{}: {} (start at {})",
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
            "`BackupPC` backup {}/{}: {}",
            backuppc.hostname, backuppc.backup_number, backuppc.size
        );
    }

    let multi = MultiProgress::new();

    let total_bar = multi.add(ProgressBar::new(size));
    total_bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {percent_precise:>7}% {msg:>65} ETA: {eta}",
        )
        // SAFETY: hardcoded template string is always valid
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
            // SAFETY: hardcoded template string is always valid
            .unwrap()
        );

        if !args.dry_run {
            let result = launch_backup(
                &context,
                state.clone(),
                &args.backuppc_pool,
                &backup,
                &mut backup_bar,
            )
            .await;
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
        let mut woodstock_backups = list_woodstock_backups(state.clone(), &excludes).await;
        woodstock_backups.sort_by_key(|backup| backup.start_time);

        let mut backuppc_backups = list_backuppc_backups(&args.backuppc_pool, &excludes);
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
            // SAFETY: hardcoded template string is always valid
            .unwrap(),
        );
        total_bar.set_message(format!("{}/{}", 0, length));
        total_bar.tick();

        for (count, backup) in woodstock_backups_to_remove.into_iter().enumerate() {
            if !args.dry_run {
                let remover = BackupRemove::new(
                    &backup.hostname,
                    backup.backup_id,
                    &context,
                    state.config.clone(),
                    state.backups.clone(),
                );

                remover.add_refcnt_to_pool().await?;

                remover.remove_refcnt_of_host().await?;

                remover.remove_backup().await?;
            }

            total_bar.inc(backup.size);
            total_bar.set_message(format!("{}/{}", count + 1, length));
        }
        total_bar.finish();
    }

    {
        let redis_url = state.config.redis_url();
        let _lock = PoolLockRedis::new_with_path(
            &redis_url,
            &state.config.path.pool_path,
            LockOperation::Import(ImportLockOperation::Cleanup),
        )
        .await?
        .lock_exclusive()
        .await?;
        PoolManager::new(state.config.clone())
            .apply_pending(&Local::now())
            .await?;
    }

    term.write_line(&format!("[4/4] {}Backups migrate", Emoji("🪄 ", "")))?;

    Ok(())
}

use std::cmp::min;

use clap::Parser;
use console::Emoji;
use console::Term;
use futures::pin_mut;
use futures::StreamExt;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use log::debug;
use log::error;
use log::info;
use log::trace;
use log::warn;
use stream_cancel::Valve;
use woodstock::config::Backups;
use woodstock::config::Configuration;
use woodstock::config::Hosts;
use woodstock::server::backup_client::BackupClient;
use woodstock::Share;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The hostname of the server
    hostname: String,

    /// The ip used to authenticate
    ip: String,

    /// The backup number (if not provided, the latest backup will be used)
    backup_number: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let term = Term::stdout();

    let configuration = Configuration::default();
    let args = Cli::parse();

    let hosts = Hosts::new(&configuration.path);
    let host_configuration = hosts.get_host(&args.hostname)?;
    let backups = Backups::new(&configuration.path);

    let backup_number = match args.backup_number {
        Some(backup_number) => backup_number,
        None => match backups.get_last_backup(&args.hostname) {
            Some(backup) => backup.number + 1,
            None => 0,
        },
    };

    let mut abort = false;

    term.write_line(&format!(
        "Backuping {} (ips = {:?})",
        &args.hostname, host_configuration.addresses,
    ))?;

    let mut client = BackupClient::new(&args.hostname, &args.ip, backup_number).await?;

    term.write_line(&format!("[1/10] {}Authenticating", Emoji("🔐 ", "")))?;

    client.authenticate(&host_configuration.password).await?;

    term.write_line(&format!(
        "[2/10] {}Create backup directory",
        Emoji("🔐 ", "")
    ))?;

    client.init_backup_directory().await?;

    let mut log_client = client.get_client();

    let (exit, valve) = Valve::new();

    let log_handle = tokio::spawn(async move {
        let logs = log_client.stream_log();
        let logs = valve.wrap(logs);
        pin_mut!(logs);

        debug!("Start getting log from clients");
        while let Some(log) = logs.next().await {
            if let Ok(log) = log {
                match log.level() {
                    woodstock::LogLevel::Verbose => {
                        trace!("[{}] {}", log.context, log.line);
                    }
                    woodstock::LogLevel::Debug => {
                        debug!("[{}] {}", log.context, log.line);
                    }
                    woodstock::LogLevel::Log => {
                        info!("[{}] {}", log.context, log.line);
                    }
                    woodstock::LogLevel::Warn => {
                        warn!("[{}] {}", log.context, log.line);
                    }
                    woodstock::LogLevel::Error => {
                        error!("[{}] {}", log.context, log.line);
                    }
                }
            } else {
                break;
            }
        }

        debug!("Client close the connection");
    });

    term.write_line(&format!("[3/10] {}Execute pre-commands", Emoji("⚙️ ", "")))?;

    if let Some(pre_commands) = host_configuration.operations.pre_commands {
        for pre_command in pre_commands {
            let reply = client.execute_command(&pre_command.command).await?;
            info!(
                "Command {} executed with code {}",
                pre_command.command, reply.code
            );
            if !reply.stdout.is_empty() {
                info!("{}", reply.stdout);
            }
            if !reply.stderr.is_empty() {
                error!("{}", reply.stderr);
            }
        }
    }

    term.write_line(&format!("[4/10] {}Upload last file list", Emoji("⬆️ ", "")))?;

    if let Some(ref operation) = host_configuration.operations.operation {
        let shares = operation
            .shares
            .iter()
            .map(|share| share.name.clone())
            .collect::<Vec<_>>();

        if let Err(err) = client.upload_file_list(shares).await {
            error!("Error uploading file list: {}", err);
            abort = true;
        }
    }

    term.write_line(&format!("[5/10] {}Download file list", Emoji("⬇️ ", "")))?;

    if let Some(ref operation) = host_configuration.operations.operation {
        let includes = operation.includes.clone().unwrap_or_default();
        let excludes = operation.excludes.clone().unwrap_or_default();

        for share in &operation.shares {
            let mut share_includes = share.includes.clone().unwrap_or_default();
            let mut share_excludes = share.excludes.clone().unwrap_or_default();

            share_includes.extend(includes.clone());
            share_excludes.extend(excludes.clone());

            let share = Share {
                includes: share_includes,
                excludes: share_excludes,
                share_path: share.name.clone(),
            };

            if !abort {
                if let Err(err) = client.download_file_list(&share, &|_| {}).await {
                    error!("Error downloading file list: {}", err);
                    abort = true;
                }
            }
        }
    }

    term.write_line(&format!("[6/10] {}Download chunks", Emoji("💾 ", "")))?;

    let bar = ProgressBar::new(client.progress().await.progress_max);
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {percent_precise}% ({bytes_per_sec}) ETA: {eta}",
        )
        .unwrap(),
    );
    if let Some(ref operation) = host_configuration.operations.operation {
        for share in &operation.shares {
            term.write_line(&format!("Downloading {}", share.name))?;

            if !abort {
                if let Err(err) = client
                    .create_backup(&share.name, &|progress| {
                        bar.set_position(min(progress.progress_current, progress.progress_max));
                    })
                    .await
                {
                    error!("Error downloading chunks: {}", err);
                    abort = true;
                }
            }
        }
    }
    bar.finish();

    term.write_line(&format!("[7/10] {}Execute post-commands", Emoji("⚙️ ", "")))?;

    if let Some(post_commands) = host_configuration.operations.post_commands {
        for post_command in post_commands {
            if !abort {
                let reply = client.execute_command(&post_command.command).await;
                match reply {
                    Ok(reply) => {
                        info!(
                            "Command {} executed with code {}",
                            post_command.command, reply.code
                        );
                        if !reply.stdout.is_empty() {
                            info!("{}", reply.stdout);
                        }
                        if !reply.stderr.is_empty() {
                            error!("{}", reply.stderr);
                        }
                    }
                    Err(err) => {
                        error!("Error executing command: {}", err);
                        abort = true;
                    }
                }
            }
        }
    }

    if let Err(err) = client.close().await {
        error!("Error closing the connection: {}", err);
        abort = true;
    }
    drop(exit);
    log_handle.abort();

    term.write_line(&format!("[8/10] {}Compact manifests", Emoji("📦 ", "")))?;

    if let Some(ref operation) = host_configuration.operations.operation {
        for share in &operation.shares {
            client.compact(&share.name).await?;
        }
    }

    term.write_line(&format!(
        "[9/10] {}Count reference of backup",
        Emoji("📏 ", "")
    ))?;

    client.count_references().await?;

    client.save_backup(!abort).await?;

    term.write_line("[10/10] Fin")?;

    Ok(())
}

//! Synchronize backups with a Woodstock server via gRPC.
//!
//! This binary provides a command-line interface to synchronize backups from a Woodstock server. It handles authentication, backup selection, progress reporting, and chunk download, using asynchronous Rust and gRPC communication.
//!
//! # Errors
//!
//! This crate will return errors if:
//! - The server cannot be reached or authentication fails.
//! - The backup number is invalid or not found.
//! - File or chunk download fails due to network or server issues.
//! - Any I/O or configuration error occurs during the backup process.

mod backup_resolver;

use std::cell::RefCell;
use std::net::SocketAddr;
use std::sync::Arc;

use clap::Parser;
use console::Emoji;
use console::Term;
use eyre::{Result, WrapErr};
use indicatif::ProgressBar;
use indicatif::ProgressDrawTarget;
use indicatif::ProgressStyle;
use tokio::sync::mpsc;
use uuid::Uuid;
use woodstock::config::Backups;
use woodstock::config::Configuration;
use woodstock::config::Context;
use woodstock::config::Hosts;
use woodstock::config::Scheduler;
use woodstock::config::DEFAULT_CHANNEL_BUFFER_SIZE;
use woodstock::config::GLOBAL_CONFIGURATION;
use woodstock::server::backup::save_machine::SaveBackupMachine;
use woodstock::server::backup::save_state::BackupExecutionState;
use woodstock::server::backup::save_state::BackupState;
use woodstock::server::client::grpc::BackupGrpcClient;

use crate::backup_resolver::resolve_backup_id;

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

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Command-line arguments for the Woodstock backup synchronization tool.
///
/// This struct defines the parameters required to connect to a Woodstock server and select a backup to synchronize.
struct Cli {
    /// The hostname of the server
    hostname: String,

    /// The ip used to authenticate
    ip: String,

    /// Optional backup identifier (UUID or sequential number) to resume an existing backup.
    /// If omitted, a new backup is created.
    backup_id: Option<String>,
}

/// Returns a human-readable message describing the current backup execution state.
///
/// # Arguments
///
/// * `state` - The current execution state of the backup process.
///
/// # Returns
///
/// A string describing the current step in the backup process, including progress and context.
fn message_from_state(state: &BackupExecutionState) -> String {
    match state {
        BackupExecutionState::Authenticate => format!("[1/10] {}Authenticating", Emoji("🔐 ", "")),
        BackupExecutionState::Waiting => format!("[0/10] {}Waiting", Emoji("⏳ ", "")),
        BackupExecutionState::Skipped => {
            format!("[0/10] {}Skipped (host unreachable)", Emoji("⏭️ ", ""))
        }
        BackupExecutionState::Cancelled => {
            format!("[0/10] {}Cancelled by user", Emoji("🛑 ", ""))
        }
        BackupExecutionState::Initialization => {
            format!("[2/10] {}Create backup directory", Emoji("🔨 ", ""))
        }
        BackupExecutionState::PreCommands(operation) => {
            format!(
                "[3/10] {}Execute pre-commands for operation: {}",
                Emoji("⚙️ ", ""),
                operation.command
            )
        }
        BackupExecutionState::DownloadFileList(share) => {
            format!(
                "[4/10] {}Download file list for share: {share}",
                Emoji("⬇️ ", ""),
            )
        }
        BackupExecutionState::DownloadChunks(share) => {
            format!(
                "[5/10] {}Download chunks for share: {share}",
                Emoji("💾 ", ""),
            )
        }
        BackupExecutionState::PostCommands(operation) => {
            format!(
                "[6/10] {}Execute post-commands for operation: {}",
                Emoji("⚙️ ", ""),
                operation.command
            )
        }
        BackupExecutionState::Compact(share) => {
            format!(
                "[7/10] {}Compact manifests for share: {share}",
                Emoji("📦 ", "")
            )
        }
        BackupExecutionState::CountReferences => {
            format!("[8/10] {}Count reference of backup", Emoji("📏 ", ""))
        }
        BackupExecutionState::AddReferencesToPool => {
            format!("[9/10] {}Add references to pool", Emoji("📥 ", ""))
        }
        BackupExecutionState::Completed => "[10/10] End".to_string(),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    tracing_subscriber::fmt::init();

    let term = Term::stdout();

    let context = Context::default();
    let args = Cli::parse();

    let state = ServiceState::default();

    let host_configuration = state.hosts.get_host(&args.hostname).await?;

    let (next_id, next_number, previous_id) = if let Some(ref id_arg) = args.backup_id {
        let (id, number) = resolve_backup_id(id_arg, &args.hostname, &state.backups).await?;
        (id, number, None)
    } else {
        match state.backups.get_last_backup(&args.hostname).await {
            Some(last) => (Uuid::now_v7(), last.number + 1, Some(last.id)),
            None => (Uuid::now_v7(), 0, None),
        }
    };

    term.write_line(&format!(
        "Backuping {} (ips = {:?})",
        &args.hostname, host_configuration.addresses,
    ))?;

    let addr = args
        .ip
        .parse::<SocketAddr>()
        .wrap_err("Invalid IP address for sync")?;
    let grpc_client = BackupGrpcClient::new(&args.hostname, &addr, state.config.clone()).await?;

    let previous_execution_state = RefCell::new(BackupExecutionState::Waiting);
    let (tx, mut rx) = mpsc::channel::<BackupState>(DEFAULT_CHANNEL_BUFFER_SIZE);

    let bar = Arc::new(ProgressBar::hidden());
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {percent_precise}% ({bytes_per_sec}) ETA: {eta}",
        )
        // SAFETY: hardcoded template string is always valid
        .unwrap(),
    );

    // Lancer une tâche pour traiter les états de sauvegarde
    let term_clone = term.clone();
    let bar_clone = Arc::clone(&bar);
    let progress_task = tokio::spawn(async move {
        while let Some(state) = rx.recv().await {
            let current_execution_state = state.execution_state.clone();
            if current_execution_state != *previous_execution_state.borrow() {
                let _ = term_clone.write_line(&message_from_state(&current_execution_state));
                *previous_execution_state.borrow_mut() = current_execution_state;

                if let BackupExecutionState::DownloadChunks(_) = &state.execution_state {
                    bar_clone.set_draw_target(ProgressDrawTarget::stderr());
                }
            }

            if let BackupExecutionState::DownloadFileList(_) = &state.execution_state {
                let mut total_progress = 0;
                for share in state.share_states.keys() {
                    if let Some(share_sate) = state.share_states.get(share) {
                        total_progress += share_sate.file_list_progression.file_size;
                    }
                }
                bar_clone.set_length(total_progress);
            }

            if let BackupExecutionState::DownloadChunks(_) = &state.execution_state {
                let mut total_progress = 0;
                for share in state.share_states.keys() {
                    if let Some(share_sate) = state.share_states.get(share) {
                        total_progress += share_sate.backup_progression.progress_current;
                    }
                }
                bar_clone.set_position(total_progress);
            }
        }
    });

    let mut client = SaveBackupMachine::new(
        grpc_client,
        &args.hostname,
        next_id,
        next_number,
        previous_id,
        &context,
        Some(tx),
        state.config.clone(),
        state.backups.clone(),
        state.hosts.clone(),
        tokio_util::sync::CancellationToken::new(),
    )
    .await?;

    // Exécuter la sauvegarde avec le canal
    Box::pin(client.execute()).await?;

    drop(client);

    // Attendre que la tâche de traitement des états se termine
    let _ = progress_task.await;

    bar.finish();

    Ok(())
}

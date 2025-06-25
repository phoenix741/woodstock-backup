//! Standalone Woodstock Backup: local backup to a local pool (no network)
//!
//! This program reads a Woodstock host configuration and performs a full backup of the local machine
//! into a local Woodstock pool, using the same format as the server. It reuses all Woodstock logic
//! (SaveBackupMachine, pool, manifest, etc.) and is suitable for use on a USB disk or local drive.

mod local_client;
mod standalone_config;

use eyre::Result;
use indicatif::{ProgressBar, ProgressStyle};
use local_client::LocalClient;
use log::info;
use tokio::sync::mpsc;
use woodstock::pool::apply_pending_refcnt_operations;
use woodstock::server::backup::save_state::{BackupExecutionState, BackupState};
use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use woodstock::config::{Backups, Configuration, Context, DEFAULT_CHANNEL_BUFFER_SIZE};
use woodstock::server::backup::save_machine::SaveBackupMachine;

use crate::standalone_config::{read_standalone_config, StandaloneClientConfig};

const HOSTNAME: &str = "localhost"; // Default host name


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
        BackupExecutionState::Authenticate => "Preparing ...".to_string(),
        BackupExecutionState::Waiting => "Preparing ...".to_string(),
        BackupExecutionState::Initialization => "Preparing ...".to_string(),
        BackupExecutionState::PreCommands(_) => "Preparing ...".to_string(),
        BackupExecutionState::DownloadFileList(share) => 
            format!("Get the file list to backup: {share}"),
        BackupExecutionState::DownloadChunks(share) => 
            format!("Aggregate chunks for share: {share}"),
        BackupExecutionState::PostCommands(operation) => 
            format!("Finalisation: {}", operation.command),
        BackupExecutionState::Compact(share) => 
            format!("Compact manifests for share: {share}"),        
        BackupExecutionState::CountReferences => 
            format!("Count reference of backup"),
        BackupExecutionState::AddReferencesToPool => 
            format!("Add references to pool"),
        BackupExecutionState::Completed => "End".to_string(),
    }
}

async fn launch_backup(
    global_configuration: &Configuration,
    client_config: &StandaloneClientConfig,
    backup_number: usize,
) -> Result<()> {
    // Create the local client (reads from local FS)
    let client = LocalClient::new(client_config);

    let previous_execution_state = RefCell::new(BackupExecutionState::Waiting);
    let (tx, mut rx) = mpsc::channel::<BackupState>(DEFAULT_CHANNEL_BUFFER_SIZE);

    let bar = Arc::new(ProgressBar::new_spinner());
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {percent_precise}% ({bytes_per_sec}) ETA: {eta} {msg}",
        )
        .unwrap(),
    );

    
    // Lancer une tâche pour traiter les états de sauvegarde
    let bar_clone = Arc::clone(&bar);
    let progress_task = tokio::spawn(async move {
        while let Some(state) = rx.recv().await {
            let current_execution_state = state.execution_state.clone();
            if current_execution_state != *previous_execution_state.borrow() {
                bar_clone.set_message(message_from_state(&current_execution_state));
                *previous_execution_state.borrow_mut() = current_execution_state;
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


    // Create the backup state machine
    let context = Context::default();
    let mut machine = SaveBackupMachine::new_from_host_configuration(
        client,
        HOSTNAME,
        backup_number,
        client_config.backup_configuration.clone(),
        &context,
        &global_configuration,
        Some(tx),
    );

    machine.execute().await?;

    drop(machine);

    // Attendre que la tâche de traitement des états se termine
    let _ = progress_task.await;

    // Compact Refcnt
    apply_pending_refcnt_operations(global_configuration, &SystemTime::now()).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    // Get the configuration path from the first command line argument
    let Some(backup_path) = std::env::args().nth(1).map(PathBuf::from) else {
        eprintln!("Usage: standalone <config_path>");
        std::process::exit(1);
    };

    let config_path = backup_path.join("config.yml");

    info!("Loading configuration from {:?}", config_path);
    let config = Configuration::from_backup_path(backup_path.clone());
    let backups = Backups::new(&config);
    let client_config = read_standalone_config(&config_path).await?;

    let last_backup = backups.get_last_backup(HOSTNAME).await;
    let backup_number = match last_backup {
        Some(last_backup) => last_backup.number + 1,
        None => 1,
    };

    launch_backup(&config, &client_config, backup_number).await?;

    Ok(())
}

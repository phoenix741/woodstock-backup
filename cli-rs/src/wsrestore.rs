//! Restore utility for Woodstock CLI.

use std::cell::RefCell;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

use clap::Parser;
use console::Emoji;
use console::Term;
use eyre::Result;
use indicatif::ProgressBar;
use indicatif::ProgressDrawTarget;
use indicatif::ProgressStyle;
use log::error;
use woodstock::config::Context;
use woodstock::config::GlobalConfiguration;
use woodstock::config::Hosts;
use woodstock::config::DEFAULT_CHANNEL_BUFFER_SIZE;
use woodstock::server::backup::restore_machine::RestoreBackupMachine;
use woodstock::server::backup::restore_machine::ShareSelection;
use woodstock::server::backup::restore_state::RestoreExecutionState;
use woodstock::server::backup::restore_state::RestoreState;
use woodstock::server::client::grpc::BackupGrpcClient;

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Command-line arguments for the restore utility.
struct Cli {
    /// The hostname of the server
    hostname: String,

    /// The ip used to authenticate
    ip: String,

    /// The backup number
    backup_number: usize,

    /// Share path
    share: String,

    /// Optional destination directory
    #[clap(long)]
    destination_directory: Option<String>,

    /// Optional filter for files
    #[clap(long)]
    filter: Option<Vec<String>>,
}

/// Format a message describing the current restore state.
fn message_from_state(state: &RestoreExecutionState) -> String {
    match state {
        RestoreExecutionState::Waiting => format!("[0/4] {}Waiting", Emoji("⏳ ", "")),
        RestoreExecutionState::Authentication => {
            format!("[1/4] {}Authentication", Emoji("🔐 ", ""))
        }
        RestoreExecutionState::Preparation(share) => {
            format!(
                "[2/4] {}Preparing restore for share: {share}",
                Emoji("⚙️ ", "")
            )
        }
        RestoreExecutionState::Restoring(share) => {
            format!(
                "[3/4] {}Restoring files for share: {share}",
                Emoji("⬇️ ", "")
            )
        }
        RestoreExecutionState::Completed => format!("[4/4] {}Completed", Emoji("✅ ", "")),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    env_logger::init();

    let term = Term::stdout();

    let context = Context::default();
    let args = Cli::parse();

    let hosts = Hosts::new(&GlobalConfiguration);
    let host_configuration = hosts.get_host(&args.hostname).await?;

    let default_path = PathBuf::from("/");
    let destination_directory = args
        .destination_directory
        .map_or(default_path, PathBuf::from);
    let list = args.filter.as_ref().map_or(Vec::new(), |list| {
        list.iter()
            .map(|filter| Path::new(filter.as_str()))
            .collect::<Vec<_>>()
    });

    term.write_line(&format!(
        "Restoring {} (ips = {:?})",
        &args.hostname, host_configuration.addresses,
    ))?;

    let grpc_client = BackupGrpcClient::new(&args.hostname, &args.ip, &GlobalConfiguration).await?;

    let previous_execution_state = RefCell::new(RestoreExecutionState::Waiting);
    let (tx, mut rx) = mpsc::channel::<RestoreState>(DEFAULT_CHANNEL_BUFFER_SIZE);

    let bar = Arc::new(ProgressBar::hidden());
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {percent_precise}% ({bytes_per_sec}) ETA: {eta}",
        )
        .unwrap(),
    );

    // Launch a task to process restore states
    let term_clone = term.clone();
    let bar_clone = Arc::clone(&bar);
    let progress_task = tokio::spawn(async move {
        while let Some(state) = rx.recv().await {
            let current_execution_state = state.execution_state.clone();
            if current_execution_state != *previous_execution_state.borrow() {
                let _ = term_clone.write_line(&message_from_state(&current_execution_state));
                *previous_execution_state.borrow_mut() = current_execution_state;

                if let RestoreExecutionState::Restoring(_) = &state.execution_state {
                    bar_clone.set_draw_target(ProgressDrawTarget::stderr());
                }
            }

            if let RestoreExecutionState::Restoring(_) = &state.execution_state {
                bar_clone.set_position(state.global_progression.progress_current);
                bar_clone.set_length(state.global_progression.progress_max);

                if state.global_progression.progress_current > 0 {
                    println!(
                        "Restore progress: {} / {}",
                        indicatif::HumanBytes(state.global_progression.progress_current),
                        indicatif::HumanBytes(state.global_progression.progress_max)
                    );
                }
            }
        }
    });

    let mut client = RestoreBackupMachine::new(
        grpc_client,
        &args.hostname,
        args.backup_number,
        &context,
        &GlobalConfiguration,
        Some(tx),
    )
    .await?;

    // Execute the restore
    let result = client
        .execute(
            &destination_directory,
            &[ShareSelection {
                share: &args.share,
                selection: list,
            }],
        )
        .await;

    drop(client);

    // Wait for the restore state processing task to finish
    let _ = progress_task.await;

    bar.finish();

    if let Err(err) = result {
        error!("Error during restore: {}", err);
        return Err(err);
    }

    Ok(())
}

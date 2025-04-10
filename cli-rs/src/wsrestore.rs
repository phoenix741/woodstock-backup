use std::path::Path;
use std::path::PathBuf;

use clap::Parser;
use console::Emoji;
use console::Term;
use eyre::Result;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use log::error;
use woodstock::config::Context;
use woodstock::config::Hosts;
use woodstock::server::backup_restore::BackupRestore;
use woodstock::server::grpc_client::BackupGrpcClient;
use woodstock::server::progression::BackupProgression;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The hostname of the server
    hostname: String,

    /// The ip used to authenticate
    ip: String,

    /// The backup number (if not provided, the latest backup will be used)
    backup_number: usize,

    /// Share path
    share: String,

    #[clap(long)]
    destination_directory: Option<String>,

    #[clap(long)]
    filter: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    env_logger::init();

    let term = Term::stdout();

    let context = Context::default();
    let args = Cli::parse();

    let hosts = Hosts::new(&context.config);
    let host_configuration = hosts.get_host(&args.hostname).await?;

    let backup_number = args.backup_number;

    let default_path = PathBuf::from("/");
    let destination_directory = args
        .destination_directory
        .map(PathBuf::from)
        .unwrap_or(default_path);
    let list = args
        .filter
        .as_ref()
        .map(|list| {
            list.iter()
                .map(|filter| Path::new(filter.as_str()))
                .collect::<Vec<_>>()
        })
        .unwrap_or(Vec::new());

    term.write_line(&format!(
        "Restoring {} (ips = {:?})",
        &args.hostname, host_configuration.addresses,
    ))?;

    let grpc_client = BackupGrpcClient::new(&args.hostname, &args.ip, &context).await?;

    let mut client = BackupRestore::new(grpc_client, &args.hostname, backup_number, &context);

    term.write_line(&format!("[1/4] {}Prepare restauration", Emoji("⚙️ ", "")))?;

    if let Err(err) = client.prepare_restauration(&args.share, &list).await {
        error!("Error restoring files: {}", err);
    }

    term.write_line(&format!("[2/4] {}Authenticating", Emoji("🔐 ", "")))?;

    client.authenticate(&host_configuration.password).await?;

    term.write_line(&format!("[3/4] {}Restore files", Emoji("⬇️ ", "")))?;

    let progress_max = client.progress().await.progress_max;
    use std::sync::{Arc, Mutex};

    let bar = Arc::new(Mutex::new(ProgressBar::new(progress_max)));
    bar.lock().unwrap().set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {percent_precise}% ({bytes_per_sec}) ETA: {eta}",
        )
        .unwrap(),
    );

    let bar_clone = Arc::clone(&bar);
    if let Err(err) = client
        .restore(
            &args.share,
            &destination_directory,
            &list,
            Box::new(move |progress: BackupProgression| {
                bar_clone
                    .lock()
                    .unwrap()
                    .set_position(progress.progress_current);
            }),
        )
        .await
    {
        error!("Error restoring files: {}", err);
    }
    bar.lock().unwrap().finish();

    if let Err(err) = client.close().await {
        error!("Error closing the connection: {}", err);
    }

    term.write_line("[4/4] Fin")?;

    Ok(())
}

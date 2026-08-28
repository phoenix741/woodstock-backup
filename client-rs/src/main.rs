//! This file contains the main entry point for the Woodstock Backup client application.
//! It defines the `WoodstockClient` struct and implements the `WoodstockClientService` trait.
//! The `WoodstockClient` struct is responsible for handling client requests and managing the client's state.
//! The `WoodstockClientService` trait defines the service interface for the Woodstock Backup client.
//! It includes methods for authentication, executing commands, refreshing the cache, and launching backups.
//! The file also includes several modules for authentication, client configuration, commands, manifest handling, and scanning.
//!
#![recursion_limit = "512"]

#[cfg(windows)]
pub mod winfw;

use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::{Parser, Subcommand};
use eyre::{Result, WrapErr};
use tokio::sync::oneshot;
use tokio::task::spawn_blocking;
use tonic::codec::CompressionEncoding;
use tonic::transport::{Identity, Server, ServerTlsConfig};
use tracing::{debug, error, info};
use woodstock::woodstock_client_service_server::WoodstockClientServiceServer;

use woodstock_client_rs::config::{get_config_path, read_config, ResolutionMode};

use woodstock_client_rs::resolve::{DirectResolveClient, ResolveClient};
use woodstock_client_rs::server::WoodstockClient;

// Ensure we have a crypto provider for rustls 0.23+
use rustls::crypto::ring;

#[cfg(feature = "mdns")]
use woodstock_client_rs::resolve::MdnsResolveClient;

/// Command-line interface options for the Woodstock client.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional path to the configuration directory.
    #[clap(long)]
    config_dir: Option<String>,

    /// Optional subcommand to execute.
    #[command(subcommand)]
    subcommand: Option<Commands>,
}

/// Available subcommands for the Woodstock client.
#[allow(clippy::enum_variant_names)]
#[derive(Subcommand, PartialEq)]
enum Commands {
    /// Install the Woodstock client as a Windows service.
    #[cfg(windows)]
    InstallService,

    /// Remove the Woodstock client Windows service.
    #[cfg(windows)]
    RemoveService,

    /// Restart the Woodstock client Windows service.
    #[cfg(windows)]
    RestartService,

    /// Run the Woodstock client as a Windows service.
    #[cfg(windows)]
    RunService,

    /// Install the Windows Firewall rule for the client.
    #[cfg(windows)]
    InstallFwRule,

    /// Remove the Windows Firewall rule for the client.
    #[cfg(windows)]
    RemoveFwRule,

    /// Update the client to the latest version.
    SelfUpdate,
}

/// Start the Woodstock client main loop.
///
/// # Errors
/// Returns an error if the client fails to start or encounters a runtime error.
async fn start_client(
    config_dir: Option<String>,
    shutdown_signal: oneshot::Receiver<()>,
) -> Result<()> {
    let config_path = config_dir.map(PathBuf::from);
    let config_path = config_path.unwrap_or_else(get_config_path);

    debug!("Config path: {}", config_path.display());
    let config_yml = config_path.join("config.yaml");
    let config = read_config(config_yml).wrap_err("Failed to read client config file")?;

    if config.auto_update {
        let config_path_update = config_path.clone();
        let config_path_clone = config_path.clone();

        info!("Auto-update is enabled, checking for updates...");
        let update_result = spawn_blocking(move || update(config_path_clone, true)).await;

        match update_result {
            Ok(Ok(())) => info!("Initial update check completed successfully"),
            Ok(Err(err)) => error!("Failed to check for updates: {}", err),
            Err(err) => error!("Update task panicked: {:?}", err),
        }

        // Start weekly update task
        info!(
            "Starting weekly update scheduler with delay: {} seconds",
            config.update_delay
        );
        tokio::spawn(async move {
            schedule_weekly_updates(config_path_update, config.update_delay).await;
        });
    }

    let root_ca = config_path.join("rootCA.pem");
    let private_key = config_path.join(format!("{}_server.key", config.hostname));
    let public_key = config_path.join(format!("{}_server.pem", config.hostname));
    let private_https_key = config_path.join(format!("{}_https.key", config.hostname));
    let public_https_key = config_path.join(format!("{}_https.pem", config.hostname));

    info!(
        "Reading certificates from config directory: {}",
        config_path.display()
    );

    let root_ca = std::fs::read_to_string(&root_ca)?;
    let public_key = std::fs::read_to_string(&public_key)?;
    let private_key = std::fs::read_to_string(&private_key)?;
    let public_https_key = std::fs::read_to_string(&public_https_key)?;
    let private_https_key = std::fs::read_to_string(&private_https_key)?;

    info!("All certificates loaded successfully");

    let addr = config.bind.parse()?;
    let woodstock_client = WoodstockClient::new(std::path::Path::new(&config_path), &config);
    let fs_accessor_shutdown = woodstock_client.fs_accessor();

    let identity = Identity::from_pem(public_key, private_key);
    let client_ca_root = tonic::transport::Certificate::from_pem(&root_ca);

    // Concat private_https_key, \n and public_https_key
    let https_pem = format!("{private_https_key}\n{public_https_key}");
    let https_identity = reqwest::Identity::from_pem(https_pem.as_bytes())
        .map_err(|e| eyre::eyre!("Failed to create HTTPS identity: {}", e))?;
    let root_ca = reqwest::Certificate::from_pem(root_ca.as_bytes())?;

    info!("TLS configuration completed successfully");

    let server = Server::builder()
        // TODO: Mutualisation with grpc_client
        .http2_keepalive_interval(Some(Duration::from_secs(30)))
        .http2_keepalive_timeout(Some(Duration::from_secs(60)))
        .tcp_keepalive(Some(Duration::from_secs(30)))
        .tls_config(
            ServerTlsConfig::new()
                .identity(identity)
                .client_ca_root(client_ca_root),
        )?
        .add_service(
            WoodstockClientServiceServer::new(woodstock_client)
                .send_compressed(CompressionEncoding::Gzip)
                .accept_compressed(CompressionEncoding::Gzip),
        );

    let mut daemon: Option<Box<dyn ResolveClient>> = None;
    match config.resolution_mode {
        #[cfg(feature = "mdns")]
        ResolutionMode::Mdns => {
            info!("Initializing mDNS resolver...");
            match MdnsResolveClient::new(config.clone()).await {
                Ok(client) => {
                    info!("mDNS resolver initialized successfully");
                    daemon = Some(Box::new(client));
                }
                Err(e) => {
                    error!("Failed to initialize mDNS resolver: {}", e);
                    return Err(e);
                }
            }
        }
        ResolutionMode::Direct => {
            if config.server.is_none() {
                error!("Direct resolution requires a server address");
                return Err(eyre::eyre!("Direct resolution requires a server address"));
            }
            info!("Initializing direct resolver...");
            match DirectResolveClient::new(config.clone(), https_identity, root_ca).await {
                Ok(client) => {
                    info!("Direct resolver initialized successfully");
                    daemon = Some(Box::new(client));
                }
                Err(e) => {
                    error!("Failed to initialize direct resolver: {}", e);
                    return Err(e);
                }
            }
        }
        ResolutionMode::None => {
            info!("No resolver configured (resolution disabled)");
        }
    }

    server
        .serve_with_shutdown(addr, async {
            info!("Waiting for shutdown signal...");
            shutdown_signal.await.ok();

            info!("Shutdown signal received - beginning graceful shutdown");

            if let Some(daemon) = daemon {
                info!("Shutting down daemon resolver...");
                daemon.shutdown().await;
                info!("Daemon resolver shutdown complete");
            }

            info!("Cleaning up active snapshots before shutdown...");
            {
                let mut fs = fs_accessor_shutdown.write().await;
                if let Err(err) = fs.cleanup_all_snapshots().await {
                    error!("Failed to clean up snapshots during shutdown: {}", err);
                } else {
                    info!("All snapshots cleaned up before shutdown");
                }
            }

            info!("Graceful context shutdown complete");
        })
        .await?;

    Ok(())
}

mod updater;
#[cfg(windows)]
pub mod winfirewall;
#[cfg(windows)]
pub mod winserv;
use updater::{schedule_weekly_updates, update};

#[tokio::main]
async fn main() -> Result<()> {
    // Install the global crypto provider for rustls 0.23+
    // We ignore the error if it is already installed.
    let _ = ring::default_provider().install_default();

    color_eyre::install()?;

    let args = Cli::parse();

    let config_path = args.config_dir.as_ref().map(PathBuf::from);
    let config_path = config_path.unwrap_or_else(get_config_path);

    // Initialize platform-specific logging first. Kept alive for the rest of
    // `main`: on Windows it owns the file-appender's background writer
    // thread, which flushes and stops when this guard drops at process exit.
    let _log_guard = setup_platform_logging(&config_path)?;

    let config_yml = config_path.join("config.yaml");
    let config = read_config(config_yml).wrap_err("Failed to read Woodstock config")?;
    info!("Woodstock client started for: {}", config.hostname);

    match args.subcommand {
        #[cfg(windows)]
        Some(Commands::InstallService) => {
            winfirewall::add_firewall_rule(&config.bind)?;
            winserv::install_service(args.config_dir)?;
        }

        #[cfg(windows)]
        Some(Commands::RemoveService) => {
            winfirewall::remove_firewall_rule()?;
            winserv::uninstall_service()?;
        }

        #[cfg(windows)]
        Some(Commands::RestartService) => {
            winserv::restart_service()?;
        }

        #[cfg(windows)]
        Some(Commands::RunService) => {
            winserv::run()?;
        }

        #[cfg(windows)]
        Some(Commands::InstallFwRule) => {
            winfirewall::add_firewall_rule(&config.bind)?;
        }

        #[cfg(windows)]
        Some(Commands::RemoveFwRule) => {
            winfirewall::remove_firewall_rule()?;
        }

        Some(Commands::SelfUpdate) => {
            let _ = spawn_blocking(move || {
                let result = update(config_path, false);
                if let Err(err) = result {
                    println!("Failed to update: {err}");
                }
            })
            .await;
        }
        None => {
            let (signal_tx, signal_rx) = oneshot::channel::<()>();

            tokio::spawn(async move {
                #[cfg(unix)]
                {
                    let mut sigterm = tokio::signal::unix::signal(
                        tokio::signal::unix::SignalKind::terminate(),
                    )
                    .expect("Failed to install SIGTERM handler");

                    tokio::select! {
                        res = tokio::signal::ctrl_c() => {
                            if let Err(e) = res {
                                error!("Failed to watch for Ctrl-C signal: {e}");
                            }
                            info!("SIGINT received, shutting down");
                        }
                        _ = sigterm.recv() => {
                            info!("SIGTERM received, shutting down");
                        }
                    }
                }

                #[cfg(not(unix))]
                {
                    if let Err(e) = tokio::signal::ctrl_c().await {
                        error!("Failed to watch for Ctrl-C signal: {e}");
                    }
                    info!("Ctrl-C received, shutting down");
                }

                if signal_tx.send(()).is_err() {
                    error!("Failed to send shutdown signal: shutdown receiver was dropped");
                }
            });

            start_client(args.config_dir, signal_rx).await?;

            // Force the server to stop
            std::process::exit(0);
        }
    }

    Ok(())
}

/// Platform-specific logging configuration using tracing_subscriber
///
/// Configures structured logging via tracing. On Linux (systemd), logs go to stdout
/// and are automatically captured by journald. On Windows, logs go to stdout and
/// can be captured by the Windows event system.
#[cfg(not(windows))]
fn setup_platform_logging(
    _config_path: &Path,
) -> Result<Option<tracing_appender::non_blocking::WorkerGuard>> {
    let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| tracing_subscriber::EnvFilter::try_new(&log_level))
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_level(true)
        .init();

    info!("Logging initialized - platform logging ready");
    Ok(None)
}

/// Platform-specific logging configuration for Windows
///
/// Configures three output sinks:
/// - `stdout` (via `tracing_subscriber::fmt`) for interactive/debug sessions
/// - Windows ETW (via `tracing_etw`) visible in the Event Viewer and WPA/PerfView
/// - a daily rolling file under `<config_dir>/logs/`, same pattern as
///   `server-rs::logger`. `stdout` is lost for the `restart-service` helper
///   (spawned with `DETACHED_PROCESS`, no console) and ETW is only captured
///   by an actively-listening trace session, so the file sink is the only
///   sink that reliably survives to be inspected after the fact.
#[cfg(windows)]
fn setup_platform_logging(
    config_path: &Path,
) -> Result<Option<tracing_appender::non_blocking::WorkerGuard>> {
    use tracing_subscriber::prelude::*;

    let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| tracing_subscriber::EnvFilter::try_new(&log_level))
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_level(true);

    let etw_layer = tracing_etw::LayerBuilder::new("Woodstock Backup")
        .build()
        .map_err(|e| eyre::eyre!("Failed to initialize ETW logging: {e}"))?;

    let file_appender = tracing_appender::rolling::daily(config_path.join("logs"), "client.log");
    let (file_appender, guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_ansi(false)
        .with_writer(file_appender);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(etw_layer)
        .with(file_layer)
        .init();

    info!("Woodstock Client Service started - logging initialized");
    Ok(Some(guard))
}

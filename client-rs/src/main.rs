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
use eyre::Result;
use log::{debug, error, info};
use self_update::cargo_crate_version;
use tokio::sync::oneshot;
use tokio::task::spawn_blocking;
use tokio::time::{interval_at, Instant};
use tonic::codec::CompressionEncoding;
use tonic::transport::{Identity, Server, ServerTlsConfig};
use woodstock::woodstock_client_service_server::WoodstockClientServiceServer;

// Platform-specific logging imports
#[cfg(windows)]
use winlog;

use woodstock_client_rs::config::{get_config_path, read_config, ResolutionMode};

use woodstock_client_rs::resolve::{DirectResolveClient, ResolveClient};
use woodstock_client_rs::server::WoodstockClient;

#[cfg(windows)]
const WINLOG_NAME: &str = "Woodstock Backup";

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
    let config = read_config(config_yml).expect("Failed to read config");

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

            info!("Graceful context shutdown complete");
        })
        .await?;

    Ok(())
}

#[cfg(windows)]
pub mod winfirewall {
    use crate::winfw::{
        create_firewall_rule, delete_firewall_rule, rule_exists, Actions, FwRule, Protocols,
    };
    use eyre::Result;
    use std::net::SocketAddr;
    use woodstock::config::DEFAULT_PORT;

    fn get_port_from_address(address: &str) -> u16 {
        match address.parse::<SocketAddr>() {
            Ok(socket_addr) => socket_addr.port(),
            Err(_) => DEFAULT_PORT,
        }
    }

    pub fn add_firewall_rule(bind: &str) -> Result<()> {
        let port = get_port_from_address(bind);

        // Règle pour autoriser le trafic TCP entrant sur le port spécifique
        let tcp_rule_name = "Woodstock Client Daemon TCP";
        if rule_exists(tcp_rule_name)? {
            delete_firewall_rule(tcp_rule_name)?;
        }

        let tcp_rule = FwRule {
            name: tcp_rule_name.to_string(),
            description: format!("Allow incoming TCP traffic on port {}", port),
            local_ports: port.to_string(),
            protocol: Protocols::Tcp,
            action: Actions::Allow,
            enabled: true,
            ..FwRule::default()
        };
        create_firewall_rule(&tcp_rule)?;

        // Règle pour autoriser le trafic UDP entrant et sortant sur le port mDNS (5353)
        let udp_rule_name = "Woodstock Client Daemon mDNS";
        if rule_exists(udp_rule_name)? {
            delete_firewall_rule(udp_rule_name)?;
        }

        let udp_rule = FwRule {
            name: udp_rule_name.to_string(),
            description: "Allow incoming and outgoing UDP traffic on port 5353 for mDNS"
                .to_string(),
            local_ports: "5353".to_string(),
            protocol: Protocols::Udp,
            action: Actions::Allow,
            enabled: true,
            ..FwRule::default()
        };
        create_firewall_rule(&udp_rule)?;

        Ok(())
    }

    pub fn remove_firewall_rule() -> Result<()> {
        // Supprimer la règle TCP
        let tcp_rule_name = "Woodstock Client Daemon TCP";
        delete_firewall_rule(tcp_rule_name)?;

        // Supprimer la règle UDP
        let udp_rule_name = "Woodstock Client Daemon mDNS";
        delete_firewall_rule(udp_rule_name)?;

        Ok(())
    }
}

#[cfg(windows)]
pub mod winserv {
    use crate::{start_client, Cli};
    use clap::Parser;
    use log::{error, info};
    use std::{
        ffi::OsString,
        sync::{Arc, Mutex},
        thread::sleep,
        time::{Duration, Instant},
    };
    use tokio::sync::oneshot;
    use windows_service::{
        define_windows_service,
        service::{
            ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl,
            ServiceExitCode, ServiceInfo, ServiceStartType, ServiceState, ServiceStatus,
            ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
        service_dispatcher,
        service_manager::{ServiceManager, ServiceManagerAccess},
        Result,
    };
    use windows_sys::Win32::Foundation::ERROR_SERVICE_DOES_NOT_EXIST;

    const SERVICE_NAME: &str = "woodstock_client_daemon";
    const SERVICE_DISPLAY_NAME: &str = "Woodstock Client Daemon";
    const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

    pub fn run() -> Result<()> {
        service_dispatcher::start(SERVICE_NAME, ffi_service_main)
    }

    define_windows_service!(ffi_service_main, woodstock_service_main);

    pub fn woodstock_service_main(_arguments: Vec<OsString>) {
        let args = Cli::parse();
        let config_dir = args.config_dir;

        info!(
            "Starting Woodstock service with config_dir: {:?}",
            config_dir
        );

        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                error!("Failed to create Tokio runtime: {:?}", e);
                return;
            }
        };

        rt.block_on(async {
            if let Err(e) = run_service(config_dir).await {
                error!("Service error: {:?}", e);
            } else {
                info!("Service completed successfully");
            }
        });
    }

    pub async fn run_service(config_dir: Option<String>) -> eyre::Result<()> {
        // Create a channel to be able to poll a stop event from the service worker loop.
        let (signal_tx, signal_rx) = oneshot::channel::<()>();
        let signal_tx = Arc::new(Mutex::new(Some(signal_tx)));

        // Define system service event handler that will be receiving service events.
        let event_handler = {
            let signal_tx = Arc::clone(&signal_tx);
            move |control_event| -> ServiceControlHandlerResult {
                match control_event {
                    // Notifies a service to report its current status information to the service
                    // control manager. Always return NoError even if not implemented.
                    ServiceControl::Interrogate => {
                        info!("Service control: Interrogate received");
                        ServiceControlHandlerResult::NoError
                    }

                    // Handle stop
                    ServiceControl::Stop => {
                        info!("Service control: Stop signal received - initiating shutdown");
                        if let Ok(mut signal_tx) = signal_tx.lock() {
                            if let Some(signal_tx) = signal_tx.take() {
                                info!("Sending shutdown signal to main service");
                                if let Err(e) = signal_tx.send(()) {
                                    error!("Failed to send shutdown signal: {:?}", e);
                                } else {
                                    info!("Shutdown signal sent successfully");
                                }
                            } else {
                                info!("Shutdown signal already sent");
                            }
                        } else {
                            error!("Failed to acquire lock on shutdown signal sender");
                        }

                        ServiceControlHandlerResult::NoError
                    }

                    ServiceControl::Shutdown => {
                        info!("Service control: Shutdown signal received");
                        if let Ok(mut signal_tx) = signal_tx.lock() {
                            if let Some(signal_tx) = signal_tx.take() {
                                info!("Sending shutdown signal to main service (shutdown)");
                                let _ = signal_tx.send(());
                            }
                        }
                        ServiceControlHandlerResult::NoError
                    }

                    ServiceControl::Pause => {
                        info!("Service control: Pause received (not implemented)");
                        ServiceControlHandlerResult::NotImplemented
                    }

                    ServiceControl::Continue => {
                        info!("Service control: Continue received (not implemented)");
                        ServiceControlHandlerResult::NotImplemented
                    }

                    other => {
                        info!(
                            "Service control: Unknown control event received: {:?}",
                            other
                        );
                        ServiceControlHandlerResult::NotImplemented
                    }
                }
            }
        };

        info!("Registering service control handler...");
        let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

        info!("Setting service status to Running...");
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        info!("Service is now running, starting main client...");

        // TRAITEMENT
        start_client(config_dir, signal_rx).await?;

        info!("Main client stopped, setting service status to Stopped...");
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        info!("Service has been marked as stopped");
        Ok(())
    }

    pub fn install_service(config_dir: Option<String>) -> eyre::Result<()> {
        let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
        let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

        // This example installs the service defined in `examples/ping_service.rs`.
        // In the real world code you would set the executable path to point to your own binary
        // that implements windows service.
        let service_binary_path =
            ::std::env::current_exe().expect("Can't find the name of the executable");

        let launch_arguments = match config_dir {
            Some(dir) => vec![
                OsString::from("--config-dir"),
                OsString::from(dir),
                OsString::from("run-service"),
            ],
            None => vec![OsString::from("run-service")],
        };

        let service_info = ServiceInfo {
            name: OsString::from(SERVICE_NAME),
            display_name: OsString::from(SERVICE_DISPLAY_NAME),
            service_type: SERVICE_TYPE,
            start_type: ServiceStartType::AutoStart,
            error_control: ServiceErrorControl::Normal,
            executable_path: service_binary_path,
            launch_arguments: launch_arguments.clone(),
            dependencies: vec![],
            account_name: None, // run as System
            account_password: None,
        };
        let service = service_manager.create_service(
            &service_info,
            ServiceAccess::CHANGE_CONFIG | ServiceAccess::START,
        )?;
        service.set_description("Woodstock Backup Software Daemon")?;

        // Start the service
        service.start(&launch_arguments)?;

        Ok(())
    }

    pub fn uninstall_service() -> eyre::Result<()> {
        let manager_access = ServiceManagerAccess::CONNECT;
        let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

        let service_access =
            ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
        let service = service_manager.open_service(SERVICE_NAME, service_access)?;

        service.delete()?;
        if service.query_status()?.current_state != ServiceState::Stopped {
            service.stop()?;
        }
        drop(service);

        let start = Instant::now();
        let timeout = Duration::from_secs(5);
        while start.elapsed() < timeout {
            if let Err(windows_service::Error::Winapi(e)) =
                service_manager.open_service(SERVICE_NAME, ServiceAccess::QUERY_STATUS)
            {
                if e.raw_os_error() == Some(ERROR_SERVICE_DOES_NOT_EXIST as i32) {
                    println!("{SERVICE_NAME} is deleted.");
                    return Ok(());
                }
            }
            sleep(Duration::from_secs(1));
        }
        println!("{SERVICE_NAME} is marked for deletion.");

        Ok(())
    }

    pub fn restart_service() -> eyre::Result<()> {
        let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
        let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

        println!("Opening service...");
        info!("Opening service...");

        let service = service_manager.open_service(
            SERVICE_NAME,
            ServiceAccess::CHANGE_CONFIG
                | ServiceAccess::STOP
                | ServiceAccess::START
                | ServiceAccess::QUERY_STATUS,
        )?;

        if service.query_status()?.current_state == ServiceState::Running {
            println!("Stopping service...");
            info!("Stopping service...");

            service.stop()?;

            println!("Waiting for service to stop...");
            info!("Waiting for service to stop...");

            let start = Instant::now();
            let timeout = Duration::from_secs(5);
            while start.elapsed() < timeout {
                if service.query_status()?.current_state == ServiceState::Stopped {
                    break;
                }
                sleep(Duration::from_secs(1));
            }
        }

        println!("Starting service...");
        info!("Starting service...");

        service.start(&Vec::<OsString>::new())?;

        println!("Service restarted");
        info!("Service restarted");

        Ok(())
    }
}

/// Update the Woodstock client to the latest version.
///
/// # Errors
/// Returns an error if the update process fails.
fn update<P: AsRef<Path>>(_config_path: P, automatic: bool) -> Result<()> {
    println!("Checking for updates...");
    info!("Checking for updates...");

    let result = self_update::backends::gitea::Update::configure()
        .with_host("https://gogs.shadoware.org")
        .repo_owner("ShadowareOrg")
        .repo_name("woodstock-backup")
        .identifier("binaries")
        .bin_name("ws_client_daemon")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .no_confirm(automatic)
        .build()?
        .update()?;

    match result {
        self_update::Status::UpToDate(_) => {
            println!("Already up-to-date");
            info!("Already up-to-date");
        }
        self_update::Status::Updated(version) => {
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                use windows_sys::Win32::System::Threading::DETACHED_PROCESS;

                let config_path = _config_path.as_ref().to_str().unwrap().to_string();
                let _ = std::thread::spawn(move || {
                    let result = std::process::Command::new(std::env::current_exe().unwrap())
                        .args(["--config-dir", &config_path, "restart-service"])
                        .creation_flags(DETACHED_PROCESS)
                        .spawn();
                    if let Err(err) = result {
                        println!("Failed to restart service: {}", err);
                        info!("Failed to restart service: {}", err);
                    } else {
                        println!("Service restarted");
                        info!("Service restarted");
                    }
                });
            }

            println!("Updated to {version}");
            info!("Updated to {}", version);
        }
    }

    Ok(())
}

/// Schedule weekly updates for the Woodstock client.
///
/// # Errors
/// Returns an error if the update process fails.
async fn schedule_weekly_updates<P: AsRef<Path>>(config_path: P, update_delay: u64) {
    let duration = Duration::from_secs(update_delay);
    let mut interval = interval_at(Instant::now() + duration, duration);
    info!(
        "Weekly update scheduler started with interval of {} seconds",
        update_delay
    );

    loop {
        interval.tick().await;
        info!("Running scheduled update check...");
        if let Err(err) = update(config_path.as_ref(), true) {
            error!("Scheduled update failed: {}", err);
        } else {
            info!("Scheduled update completed successfully");
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Cli::parse();

    let config_path = args.config_dir.as_ref().map(PathBuf::from);
    let config_path = config_path.unwrap_or_else(get_config_path);

    // Initialize platform-specific logging first
    setup_platform_logging(&config_path)?;

    let config_yml = config_path.join("config.yaml");
    let config = read_config(config_yml).expect("Failed to read config");
    info!("Woodstock client started for: {}", config.hostname);

    match args.subcommand {
        #[cfg(windows)]
        Some(Commands::InstallService) => {
            winfirewall::add_firewall_rule(&config.bind)?;
            winserv::install_service(args.config_dir)?;
            winlog::register(WINLOG_NAME);
        }

        #[cfg(windows)]
        Some(Commands::RemoveService) => {
            winfirewall::remove_firewall_rule()?;
            winserv::uninstall_service()?;
            winlog::deregister(WINLOG_NAME);
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
                tokio::signal::ctrl_c().await.unwrap();
                signal_tx.send(()).unwrap();

                info!("Ctrl-C received, shutting down");
            });

            start_client(args.config_dir, signal_rx).await?;

            // Force the server to stop
            std::process::exit(0);
        }
    }

    Ok(())
}

/// Common logging format configuration
#[cfg(not(windows))]
fn create_log_dispatch() -> fern::Dispatch {
    use chrono;
    use log::LevelFilter;

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(LevelFilter::Info)
}

/// Platform-specific logging configuration following OS best practices
#[cfg(windows)]
fn setup_platform_logging(_config_path: &Path) -> Result<()> {
    // For Windows service, use Windows Event Log only
    match winlog::init(WINLOG_NAME) {
        Ok(()) => {
            println!("Windows Event Log initialized for service");
            info!("Woodstock Client Service started - logging to Windows Event Log");
        }
        Err(e) => {
            eprintln!("Failed to initialize Windows Event Log: {}", e);

            return Err(eyre::eyre!("Failed to initialize Windows Event Log: {}", e));
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn setup_platform_logging(_config_path: &Path) -> Result<()> {
    // For Linux systemd services, prefer stdout/stderr which systemd
    // automatically redirects to journald with proper metadata
    // Users can view logs with: journalctl -u woodstock-client -f

    create_log_dispatch()
        .chain(std::io::stdout())
        .apply()
        .expect("Failed to initialize logging");

    info!(
        "Linux systemd logging initialized - use 'journalctl -u woodstock-client -f' to view logs"
    );
    Ok(())
}

#[cfg(not(any(windows, target_os = "linux")))]
fn setup_platform_logging(_config_path: &Path) -> Result<()> {
    // Fallback for other platforms
    create_log_dispatch()
        .chain(std::io::stdout())
        .apply()
        .expect("Failed to initialize logging");
    Ok(())
}

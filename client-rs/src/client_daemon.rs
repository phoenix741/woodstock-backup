//! This file contains the main entry point for the Woodstock Backup client application.
//! It defines the `WoodstockClient` struct and implements the `WoodstockClientService` trait.
//! The `WoodstockClient` struct is responsible for handling client requests and managing the client's state.
//! The `WoodstockClientService` trait defines the service interface for the Woodstock Backup client.
//! It includes methods for authentication, executing commands, refreshing the cache, and launching backups.
//! The file also includes several modules for authentication, client configuration, commands, manifest handling, and scanning.
//!
#![recursion_limit = "512"]

use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, Subcommand};
use eyre::Result;
use log::{debug, error, info, LevelFilter};
use tokio::sync::oneshot;
use tonic::codec::CompressionEncoding;
use tonic::transport::{Identity, Server, ServerTlsConfig};

use woodstock::client::config::{get_config_path, read_config};
use woodstock::client::resolve::mdns_responder;
use woodstock::client::server::WoodstockClient;
use woodstock::woodstock_client_service_server::WoodstockClientServiceServer;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[clap(long)]
    config_dir: Option<String>,

    #[command(subcommand)]
    subcommand: Option<Commands>,
}

#[allow(clippy::enum_variant_names)]
#[derive(Subcommand)]
enum Commands {
    #[cfg(windows)]
    InstallService,

    #[cfg(windows)]
    RemoveService,

    #[cfg(windows)]
    RunService,
}

async fn start_client(
    config_dir: Option<String>,
    shutdown_signal: oneshot::Receiver<()>,
) -> Result<()> {
    let config_path = config_dir.map(PathBuf::from);
    let config_path = config_path.unwrap_or_else(get_config_path);

    debug!("Config path: {}", config_path.display());
    let config_yml = config_path.join("config.yaml");
    let config = read_config(config_yml).expect("Failed to read config");

    let root_ca = config_path.join("rootCA.pem");
    let private_key = config_path.join(format!("{}_server.key", config.hostname));
    let public_key = config_path.join(format!("{}_server.pem", config.hostname));

    let root_ca = std::fs::read_to_string(root_ca).expect("Failed to read rootCA.pem");
    let public_key = std::fs::read_to_string(public_key).expect("Failed to public key");
    let private_key = std::fs::read_to_string(private_key).expect("Failed to private key");

    let addr = config.bind.parse()?;
    let woodstock_client = WoodstockClient::new(std::path::Path::new(&config_path), &config);

    let identity = Identity::from_pem(public_key, private_key);
    let client_ca_root = tonic::transport::Certificate::from_pem(root_ca);

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

    let mut daemon = None;
    if config.disable_mdns {
        info!("mDNS is disabled");
    } else {
        info!("mDNS is enabled");
        daemon = Some(mdns_responder(&config)?);
    }

    server
        .serve_with_shutdown(addr, async {
            shutdown_signal.await.ok();
            if let Some(daemon) = daemon {
                if let Err(e) = daemon.shutdown() {
                    error!("Failed to shutdown mDNS daemon: {}", e);
                }
            }

            info!("Graceful context shutdown");
        })
        .await?;

    Ok(())
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

        info!("Starting service ");

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Err(e) = run_service(config_dir).await {
                // Handle the error, by logging or something.
                error!("Error: {:?}", e);
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
                    ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

                    // Handle stop
                    ServiceControl::Stop => {
                        if let Ok(mut signal_tx) = signal_tx.lock() {
                            if let Some(signal_tx) = signal_tx.take() {
                                signal_tx.send(()).unwrap();
                            }
                        }
                        ServiceControlHandlerResult::NoError
                    }

                    // treat the UserEvent as a stop request
                    ServiceControl::UserEvent(code) => {
                        if code.to_raw() == 130 {
                            if let Ok(mut signal_tx) = signal_tx.lock() {
                                if let Some(signal_tx) = signal_tx.take() {
                                    signal_tx.send(()).unwrap();
                                }
                            }
                        }
                        ServiceControlHandlerResult::NoError
                    }

                    _ => ServiceControlHandlerResult::NotImplemented,
                }
            }
        };

        // Register system service event handler.
        // The returned status handle should be used to report service status changes to the system.
        let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

        // Tell the system that service is running
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        // For demo purposes this service sends a UDP packet once a second.

        // TRAITEMENT
        start_client(config_dir, signal_rx).await?;

        // Tell the system that service has stopped.
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

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
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Cli::parse();
    let config_dir = args.config_dir;
    let log_path = config_dir
        .clone()
        .map_or_else(get_config_path, PathBuf::from)
        .join("client.log");

    simple_logging::log_to_file(log_path, LevelFilter::Info).expect("can't log to file");
    // env_logger::builder()
    //     .filter_level(LevelFilter::Debug)
    //     .init();

    let subcommand = args.subcommand;
    match subcommand {
        #[cfg(windows)]
        Some(Commands::InstallService) => {
            winserv::install_service(config_dir)?;
        }

        #[cfg(windows)]
        Some(Commands::RemoveService) => {
            winserv::uninstall_service()?;
        }

        #[cfg(windows)]
        Some(Commands::RunService) => {
            winserv::run()?;
        }

        #[cfg(not(windows))]
        Some(_) => {
            println!("Subcommand not supported on this platform");
        }

        None => {
            let (signal_tx, signal_rx) = oneshot::channel::<()>();

            tokio::spawn(async move {
                tokio::signal::ctrl_c().await.unwrap();
                signal_tx.send(()).unwrap();
            });

            start_client(config_dir, signal_rx).await?;
        }
    }

    Ok(())
}

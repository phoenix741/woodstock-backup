//! Windows service integration for the Woodstock client daemon.

use crate::{start_client, Cli};
use clap::Parser;
use eyre::WrapErr;
use std::{
    ffi::OsString,
    sync::{Arc, Mutex},
    thread::sleep,
    time::{Duration, Instant},
};
use tokio::sync::oneshot;
use tracing::{error, info};
use windows_service::{
    define_windows_service,
    service::{
        ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl, ServiceExitCode,
        ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
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
        ::std::env::current_exe().wrap_err("Can't find the current executable path")?;

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

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
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

/// Waits (polling) for the service to reach `target_state`, up to `timeout`.
fn wait_for_state(
    service: &windows_service::service::Service,
    target_state: ServiceState,
    timeout: Duration,
) -> eyre::Result<bool> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if service.query_status()?.current_state == target_state {
            return Ok(true);
        }
        sleep(Duration::from_secs(1));
    }
    Ok(false)
}

/// Stops then starts the service, verifying it actually comes back to
/// `Running` — retrying the start once before giving up.
///
/// # Errors
/// Returns an error if the service cannot be reached, or if it fails to
/// reach `Running` after the stop/start cycle and one retry.
pub fn restart_service() -> eyre::Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    info!("Opening service...");

    let service = service_manager.open_service(
        SERVICE_NAME,
        ServiceAccess::CHANGE_CONFIG
            | ServiceAccess::STOP
            | ServiceAccess::START
            | ServiceAccess::QUERY_STATUS,
    )?;

    if service.query_status()?.current_state == ServiceState::Running {
        info!("Stopping service for restart...");
        service.stop()?;

        info!("Waiting for service to stop...");
        if !wait_for_state(&service, ServiceState::Stopped, Duration::from_secs(5))? {
            error!("Service did not reach Stopped state within timeout, starting anyway");
        }
    }

    for attempt in 1..=2 {
        info!("Starting service (attempt {attempt}/2)...");
        if let Err(e) = service.start(&Vec::<OsString>::new()) {
            error!("service.start() failed on attempt {attempt}/2: {e:?}");
        } else if wait_for_state(&service, ServiceState::Running, Duration::from_secs(10))? {
            info!("Service restarted successfully");
            return Ok(());
        } else {
            error!(
                "Service did not reach Running state after start (attempt {attempt}/2), current state: {:?}",
                service.query_status()?.current_state
            );
        }
    }

    let final_state = service.query_status()?.current_state;
    error!("Failed to restart service after 2 attempts, final state: {final_state:?}");
    Err(eyre::eyre!(
        "Failed to restart service: did not reach Running state, final state: {final_state:?}"
    ))
}

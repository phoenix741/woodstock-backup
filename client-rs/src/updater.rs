//! Self-update logic for the Woodstock client daemon.

use eyre::Result;
use self_update::cargo_crate_version;
use std::path::Path;
use std::time::Duration;
use tokio::time::{interval_at, Instant};
use tracing::{error, info};

/// Update the Woodstock client to the latest version.
///
/// # Errors
/// Returns an error if the update process fails.
pub fn update<P: AsRef<Path>>(_config_path: P, automatic: bool) -> Result<()> {
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
            info!("Already up-to-date");
        }
        self_update::Status::Updated(version) => {
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                use windows_sys::Win32::System::Threading::DETACHED_PROCESS;

                if let Some(config_path) = _config_path.as_ref().to_str() {
                    let config_path = config_path.to_string();
                    let version = version.clone();
                    let _ = std::thread::spawn(move || {
                        let exe = match std::env::current_exe() {
                            Ok(exe) => exe,
                            Err(e) => {
                                error!("Failed to get executable path, cannot restart service for update to {version}: {e}");
                                return;
                            }
                        };
                        let result = std::process::Command::new(exe)
                            .args(["--config-dir", &config_path, "restart-service"])
                            .creation_flags(DETACHED_PROCESS)
                            .spawn();
                        match result {
                            Err(err) => {
                                error!("Failed to spawn restart-service helper for update to {version}: {err}");
                            }
                            Ok(child) => {
                                info!(
                                    "Spawned restart-service helper (pid {}) for update to {version}",
                                    child.id()
                                );
                            }
                        }
                    });
                } else {
                    error!("Cannot restart service: config path contains non-UTF-8 characters");
                }
            }

            info!("Updated to {}", version);
        }
    }

    Ok(())
}

/// Schedule weekly updates for the Woodstock client.
///
/// # Errors
/// Returns an error if the update process fails.
pub async fn schedule_weekly_updates<P: AsRef<Path>>(config_path: P, update_delay: u64) {
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

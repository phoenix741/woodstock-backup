//! `ws_console archive` — manual/administrative control of archive profiles.

use console::{style, Term};
use eyre::{eyre, Result, WrapErr};
use indicatif::HumanBytes;
use tracing::info;
use woodstock::archiving::{
    dir_sync::{diff_host_dir_archive, sync_host_dir_archive},
    tar_writer::{archive_path_for, checksum_path_for, sha256_hex_of_file, write_host_tar_archive},
    ArchiveRunStatus,
};
use woodstock::config::{ArchiveFormat, ArchivingConfig};
use woodstock::EntryType;

use super::CliServiceState;

/// Lists the configured archive profiles.
///
/// # Errors
/// Returns an error if `archiving.yml` cannot be parsed.
pub async fn list_archive_profiles(state: &CliServiceState) -> Result<()> {
    let archiving = ArchivingConfig::new(state.config.clone());
    let profiles = archiving.list_profiles().await?;

    if profiles.is_empty() {
        println!("No archive profile configured.");
        return Ok(());
    }

    for profile in profiles {
        let status = ArchiveRunStatus::load(&state.config.path.jobs_path, &profile.name)
            .await
            .unwrap_or_default();
        let last_run = status
            .last_run
            .map_or_else(|| "never".to_string(), |t| t.to_rfc3339());

        println!(
            "{} [{}] format={:?} destination={:?} schedule={:?} last_run={last_run}",
            profile.name,
            if profile.enabled {
                "enabled"
            } else {
                "disabled"
            },
            profile.format,
            profile.destination,
            profile.schedule_cron,
        );
    }

    Ok(())
}

/// Verifies a tar-family archive's checksum file against the archive
/// currently on disk.
///
/// # Errors
/// Returns an error if the profile is unknown, is not a tar-family profile,
/// has no checksum file on disk, or if the recomputed checksum doesn't
/// match.
pub async fn verify_archive_profile(
    state: &CliServiceState,
    profile_name: &str,
    hostname: &str,
) -> Result<()> {
    let archiving = ArchivingConfig::new(state.config.clone());
    let profile = archiving
        .get_profile(profile_name)
        .await?
        .ok_or_else(|| eyre!("Unknown archive profile: {profile_name}"))?;

    let archive_path = archive_path_for(&profile.destination, hostname, profile.format)
        .map_err(|_| eyre!("Profile '{profile_name}' uses the 'dir' format — verify only applies to tar-family profiles"))?;
    let checksum_path = checksum_path_for(&archive_path);

    let expected = tokio::fs::read_to_string(&checksum_path)
        .await
        .wrap_err_with(|| format!("No checksum file found at {checksum_path:?}"))?;
    let expected = expected
        .split_whitespace()
        .next()
        .ok_or_else(|| eyre!("Checksum file {checksum_path:?} is empty"))?;

    let actual = sha256_hex_of_file(&archive_path)
        .await
        .wrap_err_with(|| format!("Failed to hash {archive_path:?}"))?;

    if actual == expected {
        println!("OK  {archive_path:?}");
        Ok(())
    } else {
        println!("MISMATCH  {archive_path:?}");
        println!("  expected: {expected}");
        println!("  actual:   {actual}");
        Err(eyre!("Checksum mismatch for {archive_path:?}"))
    }
}

/// Runs an archive profile immediately, regardless of whether it is enabled
/// or due — this is both the manual `ws_console archive run` entry point and
/// the exact command a USB-hotplug udev rule would invoke.
///
/// # Errors
/// Returns an error if the profile is unknown, if no host resolves, or if
/// writing the archive for a selected host fails.
pub async fn run_archive_profile(
    state: &CliServiceState,
    profile_name: &str,
    host: Option<&str>,
) -> Result<()> {
    let archiving = ArchivingConfig::new(state.config.clone());
    let profile = archiving
        .get_profile(profile_name)
        .await?
        .ok_or_else(|| eyre!("Unknown archive profile: {profile_name}"))?;

    let hosts = match host {
        Some(host) => vec![host.to_string()],
        None => {
            let all_hosts = state.hosts.list_hosts().await?;
            profile.host_selection.resolve(&all_hosts)?
        }
    };

    if hosts.is_empty() {
        info!("No host selected for archive profile '{profile_name}'");
        return Ok(());
    }

    for hostname in hosts {
        let Some(backup) = state.backups.get_last_backup(&hostname).await else {
            info!("No completed backup for host '{hostname}', skipping");
            continue;
        };

        match profile.format {
            ArchiveFormat::Tar(_)
            | ArchiveFormat::TarGz(_)
            | ArchiveFormat::TarXz(_)
            | ArchiveFormat::TarZstd(_) => {
                let output = write_host_tar_archive(
                    &state.backups,
                    &state.config.path.pool_path,
                    &hostname,
                    &backup,
                    &profile.destination,
                    profile.format,
                    None,
                )
                .await
                .wrap_err_with(|| format!("Failed to archive host '{hostname}'"))?;

                info!("Archived {hostname} -> {:?}", output.archive_path);
                if let Some(checksum_path) = output.checksum_path {
                    info!("Checksum written to {checksum_path:?}");
                }
            }
            ArchiveFormat::Dir => {
                let output = sync_host_dir_archive(
                    &state.backups,
                    &state.config.path.pool_path,
                    &hostname,
                    &backup,
                    &profile.destination,
                    None,
                )
                .await
                .wrap_err_with(|| format!("Failed to sync host '{hostname}'"))?;

                info!(
                    "Synced {hostname} -> {:?} (+{} ~{} -{})",
                    output.destination, output.added, output.modified, output.removed
                );
            }
        }
    }

    Ok(())
}

/// Dry-runs a `dir`-mode profile's next sync for one host: prints what would
/// be added/modified/removed without touching the destination.
///
/// # Errors
/// Returns an error if the profile is unknown, is not a `dir`-format
/// profile, has no completed backup for `hostname`, or if a manifest cannot
/// be read.
pub async fn diff_archive_profile(
    state: &CliServiceState,
    profile_name: &str,
    hostname: &str,
) -> Result<()> {
    let archiving = ArchivingConfig::new(state.config.clone());
    let profile = archiving
        .get_profile(profile_name)
        .await?
        .ok_or_else(|| eyre!("Unknown archive profile: {profile_name}"))?;

    if !matches!(profile.format, ArchiveFormat::Dir) {
        return Err(eyre!(
            "Profile '{profile_name}' is not a 'dir' format profile — diff only applies there"
        ));
    }

    let backup = state
        .backups
        .get_last_backup(&hostname)
        .await
        .ok_or_else(|| eyre!("No completed backup for host '{hostname}'"))?;

    let entries = diff_host_dir_archive(&state.backups, hostname, &backup, &profile.destination)
        .await
        .wrap_err_with(|| format!("Failed to diff host '{hostname}'"))?;

    let term = Term::stdout();
    if entries.is_empty() {
        term.write_line("No changes.")?;
        return Ok(());
    }

    for journal_entry in entries {
        let entry_type = journal_entry.entry_type();
        let Some(manifest) = journal_entry.manifest else {
            continue;
        };
        let path = manifest.path();
        let size = HumanBytes(manifest.size());

        match entry_type {
            EntryType::Add => {
                term.write_line(&style(format!("+{path:?} {size}")).green().to_string())?;
            }
            EntryType::Modify => {
                term.write_line(&style(format!("*{path:?} {size}")).yellow().to_string())?;
            }
            EntryType::Remove => {
                term.write_line(&style(format!("-{path:?} {size}")).red().to_string())?;
            }
            EntryType::SnapshotInfo => {}
        }
    }

    Ok(())
}

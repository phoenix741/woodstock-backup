//! This module provides file manifest comparison and management commands for Woodstock backups.
//!
//! The actual diff engine lives in `woodstock::manifest::diff` (shared with
//! the archiving feature's incremental `dir` mode) — this module is just the
//! terminal-printing wrapper around it.
//!
//! # Errors
//!
//! Functions in this module may return errors if manifest files are missing, corrupted, or if terminal output fails.

use console::{style, Term};
use eyre::Result;
use futures::{pin_mut, StreamExt};
use indicatif::HumanBytes;
use woodstock::{manifest::generate_compare_stream, EntryType};

/// Compares two manifest files and prints the differences to the console.
///
/// # Arguments
///
/// * `manifest1` - Path to the first manifest file.
/// * `manifest2` - Path to the second manifest file.
///
/// # Errors
///
/// Returns an error if writing to the terminal fails or if manifest comparison encounters an error.
///
/// # Panics
///
/// This function does not explicitly panic.
pub async fn compare(manifest1: &str, manifest2: &str) -> Result<()> {
    let term = Term::stdout();

    let stream = generate_compare_stream(manifest1, manifest2);
    pin_mut!(stream);

    while let Some(entry) = stream.next().await {
        let entry_type = entry.entry_type();

        let Some(manifest) = entry.manifest else {
            term.write_line("entry without path")?;
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

//! This module provides file manifest comparison and management commands for Woodstock backups.
//!
//! It includes utilities for generating and printing differences between manifest files, helping users track changes between backup states.
//!
//! # Errors
//!
//! Functions in this module may return errors if manifest files are missing, corrupted, or if terminal output fails.
//!
//! # Panics
//!
//! Some functions may panic if manifest paths are invalid or if file names cannot be converted to strings.

use std::path::Path;

use async_stream::stream;
use console::{style, Term};
use eyre::Result;
use futures::{pin_mut, Stream, StreamExt};
use indicatif::HumanBytes;
use woodstock::{manifest::Manifest, EntryState, EntryType, FileManifestJournalEntry};

/// Generates a stream of file manifest journal entries by comparing two manifests.
///
/// # Arguments
///
/// * `manifest1` - Path to the first manifest file.
/// * `manifest2` - Path to the second manifest file.
///
/// # Returns
///
/// A stream of `FileManifestJournalEntry` items representing added, modified, or removed files between the two manifests.
///
/// # Errors
///
/// This function yields entries and does not return errors directly, but errors may occur when reading manifests.
///
/// # Panics
///
/// This function may panic if the manifest paths are invalid or if file names cannot be converted to strings.
pub fn generate_compare_stream(
    manifest1: &str,
    manifest2: &str,
) -> impl Stream<Item = FileManifestJournalEntry> {
    let manifest1 = Path::new(manifest1);
    let manifest1 = Manifest::new(
        manifest1
            .with_extension("")
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(""),
        manifest1.parent().unwrap_or_else(|| Path::new("")),
    );

    let manifest2 = Path::new(manifest2);
    let manifest2 = Manifest::new(
        manifest2
            .with_extension("")
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(""),
        manifest2.parent().unwrap_or_else(|| Path::new("")),
    );

    stream!({
        let mut index = manifest1.load_index().await;
        let stream_manifest_2 = manifest2.read_manifest_entries();
        pin_mut!(stream_manifest_2);

        while let Some(manifest) = stream_manifest_2.next().await {
            let entry = index.mark(&manifest.path);
            if let Some(entry) = entry {
                if entry.manifest.hash.ne(&manifest.hash) {
                    yield FileManifestJournalEntry {
                        manifest: Some(manifest),
                        entry_type: EntryType::Modify as i32,

                        state: EntryState::Metadata as i32,
                        state_messages: Vec::new(),

                        xfer_start: 0,
                        xfer_calculation: 0,
                        xfer_duration: 0,
                        xfer_check: 0,
                        chunk_sizes: vec![],
                        chunk_compressed_sizes: vec![],
                        snapshot_result: None,
                    };
                }
            } else {
                yield FileManifestJournalEntry {
                    manifest: Some(manifest),
                    entry_type: EntryType::Add as i32,

                    state: EntryState::Metadata as i32,
                    state_messages: Vec::new(),

                    xfer_start: 0,
                    xfer_calculation: 0,
                    xfer_duration: 0,
                    xfer_check: 0,
                    chunk_sizes: vec![],
                    chunk_compressed_sizes: vec![],
                    snapshot_result: None,
                };
            }
        }

        let remove_stream = index.walk();
        pin_mut!(remove_stream);

        for entry in remove_stream.by_ref() {
            if !entry.mark_viewed {
                yield FileManifestJournalEntry {
                    manifest: Some(entry.manifest.clone()),
                    entry_type: EntryType::Remove as i32,

                    state: EntryState::Metadata as i32,
                    state_messages: Vec::new(),

                    xfer_start: 0,
                    xfer_calculation: 0,
                    xfer_duration: 0,
                    xfer_check: 0,
                    chunk_sizes: vec![],
                    chunk_compressed_sizes: vec![],
                    snapshot_result: None,
                };
            }
        }
    })
}

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

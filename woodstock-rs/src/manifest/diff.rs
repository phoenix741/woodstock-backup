//! Diffs two file manifests into a stream of add/modify/remove journal
//! entries.
//!
//! This is the shared engine behind `ws_console compare` and the
//! incremental `dir` archiving format's "what changed since the last sync"
//! step — both need the exact same add/modify/remove logic, so it lives here
//! once instead of being duplicated between the CLI and the archiving code.

use std::path::Path;

use async_stream::stream;
use futures::{pin_mut, Stream, StreamExt};

use super::Manifest;
use crate::{EntryState, EntryType, FileManifestJournalEntry};

/// Streams the diff between `source` (the "before" state — e.g. a snapshot
/// manifest already synced to a destination) and `target` (the "after" state
/// — e.g. the backup manifest being synced to): `Add`/`Modify` entries carry
/// `target`'s manifest data, `Remove` entries carry `source`'s.
///
/// Takes owned [`Manifest`]s (cheap — just a handful of `PathBuf`s) rather
/// than references, so the returned stream isn't lifetime-bound to the
/// caller's borrows; clone before calling if you need to keep using your own
/// copies afterward.
pub fn generate_compare_stream_from_manifests(
    source: Manifest,
    target: Manifest,
) -> impl Stream<Item = FileManifestJournalEntry> {
    stream!({
        let mut index = source.load_index().await;
        let stream_target = target.read_manifest_entries();
        pin_mut!(stream_target);

        while let Some(manifest) = stream_target.next().await {
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

/// Parses a manifest file path (e.g.
/// `/var/lib/woodstock/hosts/h/<uuid>/%2Fetc.manifest`) into a [`Manifest`].
fn parse_manifest_path(path: &str) -> Manifest {
    let path = Path::new(path);
    Manifest::new(
        path.with_extension("")
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(""),
        path.parent().unwrap_or_else(|| Path::new("")),
    )
}

/// Convenience wrapper for `ws_console compare`: parses two manifest file
/// paths and diffs them. See [`generate_compare_stream_from_manifests`] for
/// the underlying engine.
pub fn generate_compare_stream(
    manifest1: &str,
    manifest2: &str,
) -> impl Stream<Item = FileManifestJournalEntry> {
    let manifest1 = parse_manifest_path(manifest1);
    let manifest2 = parse_manifest_path(manifest2);

    generate_compare_stream_from_manifests(manifest1, manifest2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::save_file;
    use crate::utils::compression::CompressionFormat;
    use crate::{FileManifest, FileManifestStat, FileManifestType};
    use futures::stream;

    fn entry(path: &str, hash: &[u8]) -> FileManifest {
        FileManifest {
            path: path.as_bytes().to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::RegularFile as i32,
                mode: 0o644,
                size: 1,
                ..Default::default()
            }),
            hash: hash.to_vec(),
            ..Default::default()
        }
    }

    async fn write_manifest(
        dir: &std::path::Path,
        name: &str,
        entries: Vec<FileManifest>,
    ) -> Manifest {
        let manifest = Manifest::new(name, dir);
        save_file(
            &manifest.manifest_path,
            stream::iter(entries),
            false,
            CompressionFormat::Zstd,
        )
        .await
        .unwrap();
        manifest
    }

    #[tokio::test]
    async fn diffs_add_modify_remove() {
        let tmp = tempfile::tempdir().unwrap();

        let source = write_manifest(
            tmp.path(),
            "source",
            vec![
                entry("unchanged.txt", b"hash-unchanged"),
                entry("modified.txt", b"hash-old"),
                entry("removed.txt", b"hash-removed"),
            ],
        )
        .await;

        let target = write_manifest(
            tmp.path(),
            "target",
            vec![
                entry("unchanged.txt", b"hash-unchanged"),
                entry("modified.txt", b"hash-new"),
                entry("added.txt", b"hash-added"),
            ],
        )
        .await;

        let diff_stream = generate_compare_stream_from_manifests(source, target);
        pin_mut!(diff_stream);

        let mut added = Vec::new();
        let mut modified = Vec::new();
        let mut removed = Vec::new();

        while let Some(journal_entry) = diff_stream.next().await {
            let path = journal_entry
                .manifest
                .as_ref()
                .map(|m| String::from_utf8_lossy(&m.path).to_string())
                .unwrap_or_default();
            match journal_entry.entry_type() {
                EntryType::Add => added.push(path),
                EntryType::Modify => modified.push(path),
                EntryType::Remove => removed.push(path),
                EntryType::SnapshotInfo => {}
            }
        }

        assert_eq!(added, vec!["added.txt".to_string()]);
        assert_eq!(modified, vec!["modified.txt".to_string()]);
        assert_eq!(removed, vec!["removed.txt".to_string()]);
    }

    #[tokio::test]
    async fn no_diff_when_manifests_are_identical() {
        let tmp = tempfile::tempdir().unwrap();
        let entries = vec![entry("a.txt", b"hash-a"), entry("b.txt", b"hash-b")];

        let source = write_manifest(tmp.path(), "source", entries.clone()).await;
        let target = write_manifest(tmp.path(), "target", entries).await;

        let diff_stream = generate_compare_stream_from_manifests(source, target);
        pin_mut!(diff_stream);

        assert!(diff_stream.next().await.is_none());
    }
}

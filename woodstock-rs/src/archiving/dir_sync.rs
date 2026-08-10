//! Incremental `dir` archiving format: mirrors a backup's share/file tree
//! onto a plain destination directory, touching only what actually changed
//! since the last sync.
//!
//! Each share keeps a snapshot manifest at the destination root
//! (`<dest>/<hostname>/<mangled_share>.manifest`) recording the state that
//! was last successfully synced there. On each run, that snapshot is diffed
//! against the backup's current share manifest (via
//! [`crate::manifest::generate_compare_stream_from_manifests`]) and only the
//! resulting `Add`/`Modify`/`Remove` entries are applied — no full re-copy.

use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;

use eyre::Result;
use futures::{pin_mut, stream, Stream, StreamExt};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use super::archive_reader_worker_count;
use super::fs_materialize;
use super::tar_writer::ArchiveProgressCounters;
use crate::config::{Backup, Backups, DEFAULT_CHANNEL_BUFFER_SIZE};
use crate::manifest::{generate_compare_stream_from_manifests, Manifest};
use crate::proto::save_file;
use crate::utils::compression::CompressionFormat;
use crate::utils::path::mangle;
use crate::{EntryType, FileManifest, FileManifestJournalEntry, FileManifestType};

/// Summary of one `dir`-mode sync run for a host.
#[derive(Debug, Clone, Default)]
pub struct DirSyncOutput {
    pub destination: PathBuf,
    pub added: usize,
    pub modified: usize,
    pub removed: usize,
    /// Entries that failed to materialize/remove and were `warn!`-logged —
    /// when non-zero for a share, that share's snapshot manifest was left
    /// untouched (see [`sync_host_dir_archive`]) so the next run retries
    /// them instead of silently treating them as done.
    pub skipped: usize,
    /// Whether the run stopped early because of a user cancel — some shares
    /// may not have been processed at all.
    pub cancelled: bool,
}

/// Snapshot manifest path for `share` at `dest_root` — also used by
/// `ws_console archive diff` to dry-run the same comparison.
#[must_use]
pub fn snapshot_manifest(dest_root: &Path, share: &str) -> Manifest {
    Manifest::new(&mangle(share), dest_root)
}

/// One `Add`/`Modify` entry dispatched to a [`materialize_lane`] worker.
struct MaterializeTask {
    dest_path: PathBuf,
    entry_type: EntryType,
    manifest_entry: FileManifest,
}

/// Outcome of one `Add`/`Modify` entry, reported back to
/// [`aggregate_materialize_results`] — including a skip (failed,
/// `warn!`-logged entry), unlike before: the caller needs the skip count to
/// decide whether the destination actually matches the backup manifest
/// (see [`sync_host_dir_archive`]'s snapshot-commit gating).
enum MaterializeOutcome {
    Applied {
        entry_type: EntryType,
        dir_permission: Option<(PathBuf, u32)>,
    },
    Skipped,
}

/// Drains `diff_stream`, round-robining `Add`/`Modify` entries across
/// `dispatch_txs` (one channel per [`materialize_lane`] worker) and applying
/// `Remove` entries immediately, inline, on this task. Returns
/// `(removed, remove_failed)`.
///
/// Order between `Add`/`Modify` entries never matters (each targets a
/// distinct path — the diff generator never emits two of them for the same
/// path in one run), so plain round-robin is enough, no path-based routing
/// needed.
///
/// `Remove` entries are handled here rather than fanned out to a lane too —
/// not just because they're cheap (no pool reads/decompression, nothing to
/// parallelize) but because it's what keeps this safe with no extra
/// coordination: a path that appears as `Remove` in a given diff run can
/// never also be the target of an `Add`/`Modify` in that *same* run — if
/// anything were still present under it in the target manifest, that
/// subtree wouldn't be "removed" at all, it just wouldn't appear in the
/// diff. So `Remove`'s `fs_materialize::remove_entry` (which recurses via
/// `remove_dir_all` for a directory) never races a lane materializing
/// something the diff still considers live. That argument only holds if
/// removes are applied on a single task rather than also fanned out — this
/// function is that single task.
///
/// Stops picking up new entries as soon as `cancel_token` fires — already
/// dispatched `Add`/`Modify` tasks still drain through their lanes (see
/// [`materialize_lane`]), but nothing new starts.
async fn dispatch_entries(
    diff_stream: impl Stream<Item = FileManifestJournalEntry>,
    share_root: PathBuf,
    dispatch_txs: Vec<mpsc::Sender<MaterializeTask>>,
    hostname: String,
    share: String,
    cancel_token: CancellationToken,
) -> (usize, usize) {
    pin_mut!(diff_stream);
    let mut next_worker = 0usize;
    let mut removed = 0usize;
    let mut remove_failed = 0usize;

    while let Some(journal_entry) = diff_stream.next().await {
        if cancel_token.is_cancelled() {
            break;
        }

        let entry_type = journal_entry.entry_type();
        let Some(manifest_entry) = journal_entry.manifest else {
            continue;
        };
        let dest_path = share_root.join(manifest_entry.path());

        match entry_type {
            EntryType::Add | EntryType::Modify => {
                let target = &dispatch_txs[next_worker % dispatch_txs.len()];
                next_worker += 1;
                let task = MaterializeTask {
                    dest_path,
                    entry_type,
                    manifest_entry,
                };
                if target.send(task).await.is_err() {
                    // That lane died (panicked) — its `JoinHandle` will
                    // surface the panic once joined; nothing more to
                    // usefully dispatch for this share.
                    return (removed, remove_failed);
                }
            }
            EntryType::Remove => {
                if let Err(err) = fs_materialize::remove_entry(&dest_path).await {
                    warn!(
                        "Skipping removal of {dest_path:?} in dir archive for {hostname} \
                         (share {share}): {err}"
                    );
                    remove_failed += 1;
                    continue;
                }
                removed += 1;
            }
            EntryType::SnapshotInfo => {}
        }
    }

    (removed, remove_failed)
}

/// One of the N parallel materialize lane workers spawned by
/// [`sync_host_dir_archive`]. Pulls `Add`/`Modify` tasks from its own
/// dispatch channel and materializes each one via
/// [`fs_materialize::materialize_entry`] — the same pool-chunk decompression
/// bottleneck `tar_writer`'s reader workers parallelize, reused here via the
/// same `FileManifest::open_from_pool` path.
///
/// Runs on its own blocking-pool thread, driven by `Handle::block_on`, for
/// the same reason `tar_writer::run_reader_worker` does: decompression is
/// CPU-bound work happening inline in an `AsyncRead::poll_read`, and running
/// it on a shared async worker thread would steal time from other tasks on
/// the same runtime.
///
/// Checked once per task, like `tar_writer::run_writer`'s own `cancel_token`
/// check: once `cancel_token` fires, every task still queued on this lane is
/// reported [`MaterializeOutcome::Skipped`] rather than materialized —
/// consistent with a real failure, so the caller's snapshot-commit gating
/// (see [`sync_host_dir_archive`]) treats a cancelled run exactly like a
/// partially-failed one.
fn materialize_lane(
    mut dispatch_rx: mpsc::Receiver<MaterializeTask>,
    pool_path: PathBuf,
    hostname: String,
    share: String,
    results_tx: mpsc::Sender<MaterializeOutcome>,
    progress: Option<Arc<ArchiveProgressCounters>>,
    cancel_token: CancellationToken,
) {
    let handle = tokio::runtime::Handle::current();
    handle.block_on(async move {
        while let Some(task) = dispatch_rx.recv().await {
            let MaterializeTask {
                dest_path,
                entry_type,
                manifest_entry,
            } = task;

            if cancel_token.is_cancelled() {
                if results_tx.send(MaterializeOutcome::Skipped).await.is_err() {
                    return;
                }
                continue;
            }

            if let Err(err) =
                fs_materialize::materialize_entry(&manifest_entry, &dest_path, &pool_path).await
            {
                warn!(
                    "Skipping {dest_path:?} in dir archive for {hostname} \
                     (share {share}): {err}"
                );
                if results_tx.send(MaterializeOutcome::Skipped).await.is_err() {
                    return; // coordinator gone — nothing more to usefully report
                }
                continue;
            }

            let dir_permission = (manifest_entry.file_mode() == FileManifestType::Directory)
                .then(|| (dest_path, manifest_entry.mode()));

            if let Some(progress) = &progress {
                progress
                    .bytes
                    .fetch_add(manifest_entry.size(), Ordering::Relaxed);
                progress.files.fetch_add(1, Ordering::Relaxed);
            }

            let outcome = MaterializeOutcome::Applied {
                entry_type,
                dir_permission,
            };
            if results_tx.send(outcome).await.is_err() {
                return; // coordinator gone — nothing more to usefully report
            }
        }
    });
}

/// Drains `results_rx` (fed by every [`materialize_lane`] worker) into the
/// `(added, modified, skipped, pending_dir_permissions)` quadruple
/// `sync_host_dir_archive` folds into its running [`DirSyncOutput`] and uses
/// to gate the snapshot commit. Runs concurrently with the lanes (via
/// `tokio::spawn`, not joined until after `dispatch_entries` returns) so the
/// results channel never backs up and blocks a lane mid-materialize.
async fn aggregate_materialize_results(
    mut results_rx: mpsc::Receiver<MaterializeOutcome>,
) -> (usize, usize, usize, Vec<(PathBuf, u32)>) {
    let mut added = 0usize;
    let mut modified = 0usize;
    let mut skipped = 0usize;
    let mut pending_dir_permissions = Vec::new();

    while let Some(outcome) = results_rx.recv().await {
        match outcome {
            MaterializeOutcome::Applied {
                entry_type,
                dir_permission,
            } => {
                match entry_type {
                    EntryType::Add => added += 1,
                    EntryType::Modify => modified += 1,
                    EntryType::Remove | EntryType::SnapshotInfo => unreachable!(
                        "dispatch_entries only ever sends Add/Modify tasks to a materialize lane"
                    ),
                }
                if let Some(dir_permission) = dir_permission {
                    pending_dir_permissions.push(dir_permission);
                }
            }
            MaterializeOutcome::Skipped => skipped += 1,
        }
    }

    (added, modified, skipped, pending_dir_permissions)
}

/// Syncs every share of `hostname`'s `backup` into
/// `<destination_dir>/<hostname>/<share>/...`, applying only the diff
/// between each share's snapshot manifest and its current backup manifest.
///
/// `progress`, if given, has its `bytes`/`files` counters bumped by each
/// `Add`/`Modify` entry as it is materialized — the same
/// [`ArchiveProgressCounters`] type [`crate::archiving::tar_writer::write_host_tar_archive`]
/// uses, so `workers.rs` can track both archive formats through one counter
/// without a per-format special case.
///
/// `cancel_token` is checked per-entry inside [`dispatch_entries`] and
/// [`materialize_lane`], like `tar_writer::write_host_tar_archive` — a
/// cancel stops the share currently syncing without waiting for the whole
/// host, and no further share is started.
///
/// # Errors
/// Returns an error if a manifest cannot be read, or if applying a change to
/// the destination filesystem fails. On error, the snapshot manifest for the
/// share being processed is left untouched, so the next run re-diffs from
/// the last known-good state rather than a half-applied one. The same is
/// true, without an `Err`, whenever any entry was skipped or the run was
/// cancelled (see [`DirSyncOutput::skipped`]/[`DirSyncOutput::cancelled`]):
/// the snapshot commit for that share is skipped so the next run retries
/// the entries that never actually landed on disk.
pub async fn sync_host_dir_archive(
    backups: &Backups,
    pool_path: &Path,
    hostname: &str,
    backup: &Backup,
    destination_dir: &Path,
    progress: Option<Arc<ArchiveProgressCounters>>,
    cancel_token: CancellationToken,
) -> Result<DirSyncOutput> {
    let dest_root = destination_dir.join(hostname);
    let share_paths = backups.get_backup_share_paths(hostname, backup.id).await;

    let mut output = DirSyncOutput {
        destination: dest_root.clone(),
        ..Default::default()
    };

    for share in &share_paths {
        if cancel_token.is_cancelled() {
            output.cancelled = true;
            break;
        }

        let share_root = dest_root.join(crate::utils::path::safe_share_prefix(share));
        let backup_manifest = backups.get_manifest(hostname, backup.id, share);
        let snapshot = snapshot_manifest(&dest_root, share);

        let diff_stream =
            generate_compare_stream_from_manifests(snapshot.clone(), backup_manifest.clone());

        // `Add`/`Modify` entries (pool reads + decompression, the actual
        // bottleneck — see `materialize_lane`) fan out across N parallel
        // lanes; `Remove` entries are applied inline by `dispatch_entries`
        // itself. See `dispatch_entries`'s doc comment for why splitting the
        // work this way (rather than partitioning everything, `Remove`
        // included, across the same lanes) is what keeps this safe without
        // any extra path-based coordination.
        let worker_count = archive_reader_worker_count();
        let (results_tx, results_rx) = mpsc::channel(DEFAULT_CHANNEL_BUFFER_SIZE);

        let mut dispatch_txs = Vec::with_capacity(worker_count);
        let mut lane_handles = Vec::with_capacity(worker_count);
        for _ in 0..worker_count {
            let (dispatch_tx, dispatch_rx) = mpsc::channel(DEFAULT_CHANNEL_BUFFER_SIZE);
            dispatch_txs.push(dispatch_tx);
            lane_handles.push(tokio::task::spawn_blocking({
                let pool_path = pool_path.to_path_buf();
                let hostname = hostname.to_string();
                let share = share.clone();
                let results_tx = results_tx.clone();
                let progress = progress.clone();
                let cancel_token = cancel_token.clone();
                move || {
                    materialize_lane(
                        dispatch_rx,
                        pool_path,
                        hostname,
                        share,
                        results_tx,
                        progress,
                        cancel_token,
                    )
                }
            }));
        }
        drop(results_tx); // the coordinator's `results_rx` closes once every lane above finishes

        let aggregate_handle = tokio::spawn(aggregate_materialize_results(results_rx));

        let (removed, remove_failed) = dispatch_entries(
            diff_stream,
            share_root,
            dispatch_txs,
            hostname.to_string(),
            share.clone(),
            cancel_token.clone(),
        )
        .await;

        for lane_joined in futures::future::join_all(lane_handles).await {
            lane_joined?; // propagate a lane-thread panic
        }
        let (added, modified, skipped, mut pending_dir_permissions) = aggregate_handle.await?;
        let skipped = skipped + remove_failed;

        output.added += added;
        output.modified += modified;
        output.removed += removed;
        output.skipped += skipped;
        if cancel_token.is_cancelled() {
            output.cancelled = true;
        }

        // Deepest first, so an ancestor's permissions are only tightened
        // after every path underneath it has already been created. Safe to
        // run only now, after every lane above has finished (`join_all`
        // returned): every `Add`/`Modify` in this share — from every lane —
        // has already been materialized.
        pending_dir_permissions
            .sort_by_key(|(path, _)| std::cmp::Reverse(path.components().count()));
        for (path, mode) in pending_dir_permissions {
            fs_materialize::set_directory_permissions(&path, mode).await;
        }

        // The destination only actually matches `backup_manifest` for this
        // share if every entry was applied — atomically replace the
        // snapshot with a copy of it so the next run diffs from this
        // known-good state. If anything was skipped or cancelled, leave the
        // existing snapshot untouched so the next run's diff still sees
        // (and retries) whatever never actually landed on disk, instead of
        // silently and permanently treating it as done.
        if skipped == 0 && !cancel_token.is_cancelled() {
            let entries = backup_manifest.read_manifest_entries_to_end().await?;
            save_file(
                &snapshot.manifest_path,
                stream::iter(entries),
                true,
                CompressionFormat::Zstd,
            )
            .await?;
        } else {
            warn!(
                "dir archive for {hostname} (share {share}): snapshot not updated \
                 ({skipped} entr{plural} skipped, cancelled={}) — next run will retry",
                cancel_token.is_cancelled(),
                plural = if skipped == 1 { "y" } else { "ies" }
            );
        }

        if cancel_token.is_cancelled() {
            break;
        }
    }

    info!(
        "dir-mode archive sync for {hostname} -> {:?}: +{} ~{} -{} (skipped {}{})",
        output.destination,
        output.added,
        output.modified,
        output.removed,
        output.skipped,
        if output.cancelled { ", cancelled" } else { "" }
    );

    Ok(output)
}

/// Computes what a [`sync_host_dir_archive`] run would change, without
/// applying it — used for `ws_console archive diff` / dry-runs.
///
/// # Errors
/// Returns an error if a manifest cannot be read.
pub async fn diff_host_dir_archive(
    backups: &Backups,
    hostname: &str,
    backup: &Backup,
    destination_dir: &Path,
) -> Result<Vec<FileManifestJournalEntry>> {
    let dest_root = destination_dir.join(hostname);
    let share_paths = backups.get_backup_share_paths(hostname, backup.id).await;

    let mut entries = Vec::new();
    for share in &share_paths {
        let backup_manifest = backups.get_manifest(hostname, backup.id, share);
        let snapshot = snapshot_manifest(&dest_root, share);

        let diff_stream = generate_compare_stream_from_manifests(snapshot, backup_manifest);
        pin_mut!(diff_stream);
        while let Some(entry) = diff_stream.next().await {
            entries.push(entry);
        }
    }

    Ok(entries)
}

/// Sum of `Add`/`Modify` entry sizes a [`sync_host_dir_archive`] run for
/// `hostname`/`backup` would write — used to derive `dir` mode's
/// `progress_max` upfront, since (unlike tar-family) `Backup::file_size`
/// would count the whole backup rather than just the delta this run
/// actually writes.
///
/// # Errors
/// Returns an error if a manifest cannot be read.
pub async fn dir_diff_total_size(
    backups: &Backups,
    hostname: &str,
    backup: &Backup,
    destination_dir: &Path,
) -> Result<u64> {
    let entries = diff_host_dir_archive(backups, hostname, backup, destination_dir).await?;
    let total = entries
        .into_iter()
        .filter(|entry| matches!(entry.entry_type(), EntryType::Add | EntryType::Modify))
        .filter_map(|entry| entry.manifest)
        .map(|manifest| manifest.size())
        .sum();
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BackupStatus, Configuration, ShareRecord, ShareSnapshotMethod};
    use crate::pool::PoolChunkWrapper;
    use crate::{ChunkAlgorithm, FileManifest, FileManifestStat, FileManifestType};
    use chrono::Local;
    use std::sync::Arc;
    use uuid::Uuid;

    fn fake_backup(id: Uuid) -> Backup {
        Backup {
            id,
            number: 1,
            status: BackupStatus::Completed,
            start_date: Local::now(),
            end_date: Some(Local::now()),
            error_count: 0,
            error_message: None,
            file_count: 0,
            new_file_count: 0,
            removed_file_count: 0,
            modified_file_count: 0,
            existing_file_count: 0,
            file_size: 0,
            new_file_size: 0,
            modified_file_size: 0,
            existing_file_size: 0,
            compressed_file_size: 0,
            new_compressed_file_size: 0,
            modified_compressed_file_size: 0,
            existing_compressed_file_size: 0,
            speed: 0.0,
            agent_version: None,
        }
    }

    async fn write_chunk(config: &Configuration, content: &[u8]) -> Vec<u8> {
        let mut chunk = PoolChunkWrapper::new(&config.path.pool_path, None);
        let info = chunk
            .write(
                stream::once(async { Ok(content.to_vec()) }),
                b"file.txt",
                ChunkAlgorithm::Blake3,
                CompressionFormat::Zstd,
            )
            .await
            .unwrap();
        info.sha256
    }

    async fn write_backup_manifest(
        backups: &Backups,
        hostname: &str,
        backup_id: Uuid,
        share: &str,
        file_content: &[u8],
        file_hash: &[u8],
    ) {
        let dest_dir = backups.get_backup_destination_directory(hostname, backup_id);
        tokio::fs::create_dir_all(&dest_dir).await.unwrap();

        backups
            .add_backup_share_record(
                hostname,
                backup_id,
                ShareRecord {
                    path: share.to_string(),
                    snapshot_method: ShareSnapshotMethod::None,
                    snapshot_failure_reason: None,
                },
            )
            .await
            .unwrap();

        let manifest = backups.get_manifest(hostname, backup_id, share);
        let dir_entry = FileManifest {
            path: b"dir".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::Directory as i32,
                mode: 0o755,
                ..Default::default()
            }),
            ..Default::default()
        };
        let file_entry = FileManifest {
            path: b"dir/file.txt".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::RegularFile as i32,
                mode: 0o644,
                size: file_content.len() as u64,
                ..Default::default()
            }),
            chunks: vec![file_hash.to_vec()],
            hash: file_hash.to_vec(),
            ..Default::default()
        };
        save_file(
            &manifest.manifest_path,
            stream::iter(vec![dir_entry, file_entry]),
            false,
            CompressionFormat::Zstd,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn first_sync_copies_everything_then_second_sync_is_a_no_op() {
        let tmp = tempfile::tempdir().unwrap();
        let config = Arc::new(Configuration::from_backup_path(tmp.path().to_path_buf()));
        let backups = Backups::new(config.clone());
        let hostname = "fakehost";
        let backup_id = Uuid::now_v7();

        let content = b"hello world v1".to_vec();
        let hash = write_chunk(&config, &content).await;
        write_backup_manifest(&backups, hostname, backup_id, "/share", &content, &hash).await;

        let dest_dir = tmp.path().join("archive-out");
        let backup = fake_backup(backup_id);

        let first = sync_host_dir_archive(
            &backups,
            &config.path.pool_path,
            hostname,
            &backup,
            &dest_dir,
            None,
            CancellationToken::new(),
        )
        .await
        .unwrap();
        assert_eq!(first.added, 2); // "dir" + "dir/file.txt"
        assert_eq!(first.modified, 0);
        assert_eq!(first.removed, 0);

        let written = tokio::fs::read(dest_dir.join(hostname).join("share/dir/file.txt"))
            .await
            .unwrap();
        assert_eq!(written, content);

        let second = sync_host_dir_archive(
            &backups,
            &config.path.pool_path,
            hostname,
            &backup,
            &dest_dir,
            None,
            CancellationToken::new(),
        )
        .await
        .unwrap();
        assert_eq!(second.added, 0);
        assert_eq!(second.modified, 0);
        assert_eq!(second.removed, 0);
    }

    /// An entry that fails to materialize (here: its chunk was never
    /// written to the pool) must not be silently treated as done — the
    /// snapshot for that share must stay uncommitted so the next run's diff
    /// still sees it as an `Add` and retries it, instead of the file
    /// permanently missing from the dir-mode archive with no error and no
    /// retry (see PR #107 review, `sync_host_dir_archive`'s snapshot-commit
    /// gating on `DirSyncOutput::skipped`).
    #[tokio::test]
    async fn skipped_entry_leaves_snapshot_untouched_so_next_run_retries() {
        let tmp = tempfile::tempdir().unwrap();
        let config = Arc::new(Configuration::from_backup_path(tmp.path().to_path_buf()));
        let backups = Backups::new(config.clone());
        let hostname = "fakehost";
        let backup_id = Uuid::now_v7();

        // A hash that was never written to the pool: `materialize_entry`
        // fails to read it, so the file entry is skipped rather than
        // applied — unlike `write_backup_manifest`'s normal use, no
        // `write_chunk` call precedes this.
        let missing_hash = vec![0xAB; 32];
        write_backup_manifest(
            &backups,
            hostname,
            backup_id,
            "/share",
            b"never actually written to the pool",
            &missing_hash,
        )
        .await;

        let dest_dir = tmp.path().join("archive-out");
        let backup = fake_backup(backup_id);

        let first = sync_host_dir_archive(
            &backups,
            &config.path.pool_path,
            hostname,
            &backup,
            &dest_dir,
            None,
            CancellationToken::new(),
        )
        .await
        .unwrap();
        assert_eq!(first.added, 1, "the dir entry itself still materializes");
        assert_eq!(
            first.skipped, 1,
            "the file entry with a missing pool chunk is skipped, not applied"
        );
        assert!(!first.cancelled);

        // Same still-missing chunk, second run: if the snapshot had been
        // committed anyway (the bug this test guards against), the diff
        // would now see the file as already synced and never retry it.
        let second = sync_host_dir_archive(
            &backups,
            &config.path.pool_path,
            hostname,
            &backup,
            &dest_dir,
            None,
            CancellationToken::new(),
        )
        .await
        .unwrap();
        assert_eq!(
            second.skipped, 1,
            "an uncommitted snapshot means the second run retries the same skipped entry"
        );
    }

    #[tokio::test]
    async fn resync_only_touches_the_file_that_changed() {
        let tmp = tempfile::tempdir().unwrap();
        let config = Arc::new(Configuration::from_backup_path(tmp.path().to_path_buf()));
        let backups = Backups::new(config.clone());
        let hostname = "fakehost";
        let dest_dir = tmp.path().join("archive-out");

        let backup1_id = Uuid::now_v7();
        let content1 = b"hello world v1".to_vec();
        let hash1 = write_chunk(&config, &content1).await;
        write_backup_manifest(&backups, hostname, backup1_id, "/share", &content1, &hash1).await;
        sync_host_dir_archive(
            &backups,
            &config.path.pool_path,
            hostname,
            &fake_backup(backup1_id),
            &dest_dir,
            None,
            CancellationToken::new(),
        )
        .await
        .unwrap();

        let backup2_id = Uuid::now_v7();
        let content2 = b"hello world v2 (changed)".to_vec();
        let hash2 = write_chunk(&config, &content2).await;
        write_backup_manifest(&backups, hostname, backup2_id, "/share", &content2, &hash2).await;

        let resync = sync_host_dir_archive(
            &backups,
            &config.path.pool_path,
            hostname,
            &fake_backup(backup2_id),
            &dest_dir,
            None,
            CancellationToken::new(),
        )
        .await
        .unwrap();

        assert_eq!(resync.added, 0);
        assert_eq!(resync.modified, 1);
        assert_eq!(resync.removed, 0);

        let written = tokio::fs::read(dest_dir.join(hostname).join("share/dir/file.txt"))
            .await
            .unwrap();
        assert_eq!(written, content2);
    }

    /// This is the case that distinguishes a correct `Add`/`Modify` lane
    /// partition from a merely lucky one: an entire directory subtree
    /// disappearing (`old_dir/` and its 3 children, all `Remove`, the last
    /// one via `fs_materialize::remove_entry`'s `remove_dir_all`) in the
    /// *same* run as a large number of brand-new files spread across
    /// several materialize lanes (well beyond `archive_reader_worker_count`'s
    /// default of 4, so lanes are genuinely running concurrently, not just
    /// nominally parallel). If `Remove` were ever fanned out to a lane
    /// alongside `Add`/`Modify` instead of applied inline by
    /// `dispatch_entries`, or if the "removed paths never overlap
    /// concurrently-added paths" argument in its doc comment were wrong,
    /// this is the shape of bug that would surface: a new file silently
    /// missing, or a spurious removal/creation error logged and swallowed.
    #[tokio::test]
    async fn resync_removes_whole_subtree_while_concurrently_adding_many_new_files() {
        let tmp = tempfile::tempdir().unwrap();
        let config = Arc::new(Configuration::from_backup_path(tmp.path().to_path_buf()));
        let backups = Backups::new(config.clone());
        let hostname = "fakehost";
        let dest_dir = tmp.path().join("archive-out");
        let share = "/share";

        async fn write_manifest_entries(
            backups: &Backups,
            hostname: &str,
            backup_id: Uuid,
            share: &str,
            entries: Vec<FileManifest>,
        ) {
            let dest_dir = backups.get_backup_destination_directory(hostname, backup_id);
            tokio::fs::create_dir_all(&dest_dir).await.unwrap();
            backups
                .add_backup_share_record(
                    hostname,
                    backup_id,
                    ShareRecord {
                        path: share.to_string(),
                        snapshot_method: ShareSnapshotMethod::None,
                        snapshot_failure_reason: None,
                    },
                )
                .await
                .unwrap();
            let manifest = backups.get_manifest(hostname, backup_id, share);
            save_file(
                &manifest.manifest_path,
                stream::iter(entries),
                false,
                CompressionFormat::Zstd,
            )
            .await
            .unwrap();
        }

        fn dir_entry(path: &str) -> FileManifest {
            FileManifest {
                path: path.as_bytes().to_vec(),
                stats: Some(FileManifestStat {
                    file_type: FileManifestType::Directory as i32,
                    mode: 0o755,
                    ..Default::default()
                }),
                ..Default::default()
            }
        }

        async fn file_entry(config: &Configuration, path: &str, content: &[u8]) -> FileManifest {
            let mut chunk = PoolChunkWrapper::new(&config.path.pool_path, None);
            let info = chunk
                .write(
                    stream::once({
                        let bytes = content.to_vec();
                        async move { Ok(bytes) }
                    }),
                    path.as_bytes(),
                    ChunkAlgorithm::Blake3,
                    CompressionFormat::Zstd,
                )
                .await
                .unwrap();
            FileManifest {
                path: path.as_bytes().to_vec(),
                stats: Some(FileManifestStat {
                    file_type: FileManifestType::RegularFile as i32,
                    mode: 0o644,
                    size: content.len() as u64,
                    ..Default::default()
                }),
                chunks: vec![info.sha256.clone()],
                hash: info.sha256,
                ..Default::default()
            }
        }

        // Backup 1: `old_dir/` with 3 children — this whole subtree is what
        // backup 2 removes.
        let backup1_id = Uuid::now_v7();
        let mut backup1_entries = vec![dir_entry("old_dir")];
        for name in ["a.txt", "b.txt", "c.txt"] {
            backup1_entries
                .push(file_entry(&config, &format!("old_dir/{name}"), b"old content").await);
        }
        write_manifest_entries(&backups, hostname, backup1_id, share, backup1_entries).await;
        sync_host_dir_archive(
            &backups,
            &config.path.pool_path,
            hostname,
            &fake_backup(backup1_id),
            &dest_dir,
            None,
            CancellationToken::new(),
        )
        .await
        .unwrap();
        assert!(dest_dir.join(hostname).join("share/old_dir/a.txt").exists());

        // Backup 2: `old_dir/` is entirely gone; `new_dir/` shows up instead
        // with enough files to genuinely spread across several lanes.
        const NEW_FILE_COUNT: usize = 24;
        let backup2_id = Uuid::now_v7();
        let mut backup2_entries = vec![dir_entry("new_dir")];
        let mut new_contents = Vec::new();
        for i in 0..NEW_FILE_COUNT {
            let name = format!("new_dir/f{i:02}.txt");
            let content = format!("new file #{i}").into_bytes();
            backup2_entries.push(file_entry(&config, &name, &content).await);
            new_contents.push((name, content));
        }
        write_manifest_entries(&backups, hostname, backup2_id, share, backup2_entries).await;

        let resync = sync_host_dir_archive(
            &backups,
            &config.path.pool_path,
            hostname,
            &fake_backup(backup2_id),
            &dest_dir,
            None,
            CancellationToken::new(),
        )
        .await
        .unwrap();

        // 1 new dir + 24 new files added; old_dir + its 3 children removed.
        assert_eq!(resync.added, 1 + NEW_FILE_COUNT);
        assert_eq!(resync.modified, 0);
        assert_eq!(resync.removed, 4);

        let share_root = dest_dir.join(hostname).join("share");
        assert!(
            !share_root.join("old_dir").exists(),
            "old_dir and everything under it must be gone"
        );
        for (name, content) in &new_contents {
            let written = tokio::fs::read(share_root.join(name))
                .await
                .unwrap_or_else(|err| panic!("missing or unreadable {name}: {err}"));
            assert_eq!(&written, content, "content mismatch for {name}");
        }
    }

    /// Reproduces the two real-world Windows-host issues seen in production:
    /// a raw `C:\` share, which used to collapse into one oddly-named
    /// directory (`C:\`) instead of a `C/...` tree, and a WindowsApps "app
    /// execution alias" reported as a symlink with an empty target, which
    /// used to abort the whole sync via `materialize_entry`'s `?`.
    #[tokio::test]
    async fn windows_share_and_empty_symlink_do_not_abort_the_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let config = Arc::new(Configuration::from_backup_path(tmp.path().to_path_buf()));
        let backups = Backups::new(config.clone());
        let hostname = "windowshost";
        let backup_id = Uuid::now_v7();
        let share = "C:\\";

        let content = b"hello from a windows share".to_vec();
        let hash = write_chunk(&config, &content).await;

        let dest_dir = backups.get_backup_destination_directory(hostname, backup_id);
        tokio::fs::create_dir_all(&dest_dir).await.unwrap();
        backups
            .add_backup_share_record(
                hostname,
                backup_id,
                ShareRecord {
                    path: share.to_string(),
                    snapshot_method: ShareSnapshotMethod::None,
                    snapshot_failure_reason: None,
                },
            )
            .await
            .unwrap();

        let manifest = backups.get_manifest(hostname, backup_id, share);
        let file_entry = FileManifest {
            path: b"dir/file.txt".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::RegularFile as i32,
                mode: 0o644,
                size: content.len() as u64,
                ..Default::default()
            }),
            chunks: vec![hash.clone()],
            hash: hash.clone(),
            ..Default::default()
        };
        let broken_symlink_entry = FileManifest {
            path: b"WindowsApps/python3.exe".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::Symlink as i32,
                mode: 0o777,
                ..Default::default()
            }),
            symlink: vec![],
            ..Default::default()
        };
        save_file(
            &manifest.manifest_path,
            stream::iter(vec![file_entry, broken_symlink_entry]),
            false,
            CompressionFormat::Zstd,
        )
        .await
        .unwrap();

        let archive_dest_dir = tmp.path().join("archive-out");
        let result = sync_host_dir_archive(
            &backups,
            &config.path.pool_path,
            hostname,
            &fake_backup(backup_id),
            &archive_dest_dir,
            None,
            CancellationToken::new(),
        )
        .await
        .unwrap();

        // Both diff entries are processed (`added` counts entries applied,
        // not files materialized) but only the regular file actually lands
        // on disk — the broken symlink is silently skipped.
        assert_eq!(result.added, 2);

        let written = tokio::fs::read(archive_dest_dir.join(hostname).join("C/dir/file.txt"))
            .await
            .unwrap();
        assert_eq!(written, content);
    }

    /// Reproduces a real-world production failure: some captured directories
    /// (e.g. `/etc/iscsi/nodes/.../...`) have an owner-only mode with no
    /// execute bit (0o600). Applying that mode to the destination directory
    /// as soon as it is created — before its children are written — used to
    /// lock the sync out of its own directory and abort the whole run with
    /// "Permission denied". Permissions must be applied only after every
    /// entry underneath has been materialized, and must always keep at least
    /// owner rwx since this is a disaster-recovery copy, not a security
    /// mirror of the source.
    #[tokio::test]
    async fn restrictive_directory_mode_does_not_block_its_own_children() {
        let tmp = tempfile::tempdir().unwrap();
        let config = Arc::new(Configuration::from_backup_path(tmp.path().to_path_buf()));
        let backups = Backups::new(config.clone());
        let hostname = "restrictedhost";
        let backup_id = Uuid::now_v7();
        let share = "/etc";

        let content = b"top secret".to_vec();
        let hash = write_chunk(&config, &content).await;

        let dest_dir = backups.get_backup_destination_directory(hostname, backup_id);
        tokio::fs::create_dir_all(&dest_dir).await.unwrap();
        backups
            .add_backup_share_record(
                hostname,
                backup_id,
                ShareRecord {
                    path: share.to_string(),
                    snapshot_method: ShareSnapshotMethod::None,
                    snapshot_failure_reason: None,
                },
            )
            .await
            .unwrap();

        let manifest = backups.get_manifest(hostname, backup_id, share);
        let restrictive_dir_entry = FileManifest {
            path: b"iscsi/nodes".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::Directory as i32,
                mode: 0o600, // no execute bit, matches real captured data
                ..Default::default()
            }),
            ..Default::default()
        };
        let file_entry = FileManifest {
            path: b"iscsi/nodes/config".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::RegularFile as i32,
                mode: 0o600,
                size: content.len() as u64,
                ..Default::default()
            }),
            chunks: vec![hash.clone()],
            hash: hash.clone(),
            ..Default::default()
        };
        save_file(
            &manifest.manifest_path,
            stream::iter(vec![restrictive_dir_entry, file_entry]),
            false,
            CompressionFormat::Zstd,
        )
        .await
        .unwrap();

        let archive_dest_dir = tmp.path().join("archive-out");
        let result = sync_host_dir_archive(
            &backups,
            &config.path.pool_path,
            hostname,
            &fake_backup(backup_id),
            &archive_dest_dir,
            None,
            CancellationToken::new(),
        )
        .await
        .unwrap();

        assert_eq!(result.added, 2);

        let dest_dir = archive_dest_dir.join(hostname).join("etc");
        let written = tokio::fs::read(dest_dir.join("iscsi/nodes/config"))
            .await
            .unwrap();
        assert_eq!(written, content);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let dir_mode = tokio::fs::metadata(dest_dir.join("iscsi/nodes"))
                .await
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(
                dir_mode, 0o700,
                "owner execute bit must be preserved even though the source captured 0o600"
            );
        }
    }
}

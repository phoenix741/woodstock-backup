//! Writes a host's latest backup out as a single, portable tar/tar.gz/tar.xz
//! archive file, streamed directly from the content-addressable pool.
//!
//! The output uses standard tar/gzip/xz formats (readable by ordinary `tar`,
//! `gzip`, `xz` tools) — this deliberately does NOT reuse
//! [`crate::utils::compression::WoodstockCompressionWriter`], which prepends
//! a Woodstock-proprietary magic header unsuitable for portable archives.

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::task::{Context, Poll};

use async_compression::tokio::write::{GzipEncoder, XzEncoder, ZstdEncoder};
use async_compression::Level;
use bytes::Bytes;
use eyre::{eyre, Result};
use futures::StreamExt;
use tar::{Builder, EntryType as TarEntryType, Header};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt, BufWriter};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::io::{StreamReader, SyncIoBridge};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, warn};

use super::archive_reader_worker_count;
use crate::config::{ArchiveFormat, Backup, Backups, BUFFER_SIZE, DEFAULT_CHANNEL_BUFFER_SIZE};
use crate::proto::ProtobufReader;
use crate::utils::chunk_hasher::{create_chunk_hasher, ChunkHasher};
use crate::{ChunkAlgorithm, FileManifest, FileManifestType};

/// Result of a successful tar-family archive run for one host.
#[derive(Debug, Clone)]
pub struct TarArchiveOutput {
    pub archive_path: PathBuf,
    pub checksum_path: Option<PathBuf>,
    /// Final size in bytes of `archive_path`, read back from disk once the
    /// archive is fully flushed (post-compression) — distinct from the
    /// source bytes tallied in [`ArchiveProgressCounters::bytes`].
    pub archive_size: u64,
}

/// Byte and file counters shared between the reader/writer pipeline
/// ([`append_entry`]) and the caller's progress ticker. Grouped into one
/// struct (rather than two separate `Arc<AtomicU64>`s threaded through the
/// same call chain) so a caller diffing progress since a host's start
/// (`handle_archive_run`'s per-host offsets) always samples both counters
/// together with [`ArchiveProgressCounters::snapshot`], instead of repeating
/// the same "load, then subtract the starting offset" dance twice.
#[derive(Debug, Default)]
pub struct ArchiveProgressCounters {
    pub bytes: AtomicU64,
    pub files: AtomicUsize,
}

impl ArchiveProgressCounters {
    /// Current `(bytes, files)` values, each loaded with `Relaxed` ordering —
    /// approximate/live progress reporting, not a synchronization point.
    #[must_use]
    pub fn snapshot(&self) -> (u64, usize) {
        (
            self.bytes.load(Ordering::Relaxed),
            self.files.load(Ordering::Relaxed),
        )
    }
}

/// File extension (including the leading dot) used for each tar-family format.
fn extension(format: ArchiveFormat) -> Result<&'static str> {
    match format {
        ArchiveFormat::Tar(_) => Ok(".tar"),
        ArchiveFormat::TarGz(_) => Ok(".tar.gz"),
        ArchiveFormat::TarXz(_) => Ok(".tar.xz"),
        ArchiveFormat::TarZstd(_) => Ok(".tar.zst"),
        ArchiveFormat::Dir => Err(eyre!(
            "write_host_tar_archive does not support the Dir format"
        )),
    }
}

/// Each codec's own recommended default compression level, used when a
/// profile doesn't set `compressionLevel`. Not applicable to `Tar`/`Dir`.
fn default_compression_level(format: ArchiveFormat) -> i32 {
    match format {
        // zlib/gzip's own documented default (`Z_DEFAULT_COMPRESSION` maps to 6).
        ArchiveFormat::TarGz(_) => 6,
        // The `xz`/`liblzma` CLI's own documented default preset.
        ArchiveFormat::TarXz(_) => 6,
        // `ZSTD_CLEVEL_DEFAULT`.
        ArchiveFormat::TarZstd(_) => 3,
        ArchiveFormat::Tar(_) | ArchiveFormat::Dir => 0,
    }
}

/// Resolves `format`'s optional `compressionLevel` into the [`Level`] passed
/// to its codec. Each codec's `From<Level>` impl clamps out-of-range
/// `Precise` values to its own valid range (see `compression-codecs`'
/// `flate`/`xz2`/`zstd` `params.rs`), so a bad value in `archiving.yml`
/// degrades gracefully instead of failing the run.
fn resolve_compression_level(format: ArchiveFormat) -> Level {
    let requested = format.tar_options().and_then(|o| o.compression_level);
    Level::Precise(requested.unwrap_or_else(|| default_compression_level(format)))
}

/// Path a tar-family archive for `hostname` would be written to under
/// `destination_dir` for the given `format` — used by `ws_console archive
/// verify` to locate an archive without re-running the writer.
///
/// # Errors
/// Returns an error if `format` is [`ArchiveFormat::Dir`].
pub fn archive_path_for(
    destination_dir: &Path,
    hostname: &str,
    format: ArchiveFormat,
) -> Result<PathBuf> {
    let ext = extension(format)?;
    Ok(destination_dir.join(format!("{hostname}{ext}")))
}

/// Writes `hostname`'s `backup` as a single tar-family archive file under
/// `destination_dir`, optionally alongside a `sha256sum`-compatible checksum
/// file.
///
/// `progress`, if given, has its `bytes`/`files` counters bumped as each
/// entry is written — a plain [`ArchiveProgressCounters`] rather than a
/// channel, since the write itself runs on the blocking pool (see
/// [`write_tar_stream`]) and a per-file channel send would need
/// `blocking_send`, which a caller publishing progress on its own ~1s ticker
/// doesn't need.
///
/// # Errors
///
/// Returns an error if the destination cannot be created, if any manifest or
/// pool chunk cannot be read, or if the archive cannot be written.
#[allow(clippy::too_many_arguments)]
pub async fn write_host_tar_archive(
    backups: &Backups,
    pool_path: &Path,
    hostname: &str,
    backup: &Backup,
    destination_dir: &Path,
    format: ArchiveFormat,
    progress: Option<Arc<ArchiveProgressCounters>>,
    cancel_token: CancellationToken,
) -> Result<TarArchiveOutput> {
    tokio::fs::create_dir_all(destination_dir).await?;
    let archive_path = archive_path_for(destination_dir, hostname, format)?;

    let checksum_hex = match write_tar_stream(
        &archive_path,
        format,
        pool_path,
        backups,
        hostname,
        backup,
        progress,
        cancel_token,
    )
    .await
    {
        Ok(checksum_hex) => checksum_hex,
        Err(err) => {
            // The tar file was already created (and possibly partially written)
            // by the time a manifest read fails partway through — don't leave a
            // truncated archive behind, especially on removable-media destinations.
            let _ = tokio::fs::remove_file(&archive_path).await;
            // A checksum file from an earlier successful run of this host
            // would otherwise keep pointing at an archive that no longer
            // exists — routine now that a user cancel takes this same path.
            let _ = tokio::fs::remove_file(checksum_path_for(&archive_path)).await;
            return Err(err);
        }
    };

    let archive_size = tokio::fs::metadata(&archive_path).await?.len();

    let checksum_path = if let Some(hex_digest) = checksum_hex {
        Some(write_checksum_file(&archive_path, &hex_digest).await?)
    } else {
        None
    };

    Ok(TarArchiveOutput {
        archive_path,
        checksum_path,
        archive_size,
    })
}

/// Read/write buffer size used to batch reads (from a regular file entry's
/// body channel) and writes (to the compressor) on either side of the
/// sync↔async bridge. `tar::Builder`'s internal `io::copy` calls read/write
/// in small (8KB) chunks; without this, each of those chunks round-trips
/// through `SyncIoBridge::block_on` individually. Sized to match the pool's
/// native chunk read granularity (see `BUFFER_SIZE`) so a full read/tar write
/// typically crosses each bridge once instead of dozens of times.
const BRIDGE_BUFFER_SIZE: usize = 256 * 1024;

/// Depth of a regular file's dedicated body channel, in `BUFFER_SIZE` (128KB)
/// units — 16 × 128KB = 2MB of read-ahead per channel. Up to
/// [`archive_reader_worker_count`] body channels can be open at once (one per
/// parallel reader worker), so the worst-case read-ahead memory footprint is
/// `archive_reader_worker_count() * TAR_BODY_CHANNEL_CAPACITY * BUFFER_SIZE`
/// — e.g. 8MB at the default of 4 workers. Not sized defensively against the
/// archive's file count — just deep enough per worker to absorb a disk
/// round-trip's latency without starving the writer.
const TAR_BODY_CHANNEL_CAPACITY: usize = 16;

/// Depth of each per-worker dispatch channel (manifest entries waiting to be
/// picked up by a reader worker) — same rationale/value as
/// `TAR_BODY_CHANNEL_CAPACITY`: just enough that the dispatcher doesn't block
/// on every send while a worker is still busy on its current entry.
const DISPATCH_CHANNEL_CAPACITY: usize = 16;

/// A manifest entry forwarded from the reader task to the writer task.
/// `body_rx`, when present, is a channel dedicated to this one entry: the
/// reader closes it (drops the `Sender`) exactly when it reaches the real end
/// of the body (EOF or error) — end-of-entry is signaled by channel closure,
/// not by counting declared bytes (see `has_body`/`append_entry`).
struct EntryMsg {
    archive_path: PathBuf,
    entry: FileManifest,
    body_rx: Option<mpsc::Receiver<std::io::Result<Bytes>>>,
}

/// Whether `entry` needs its body streamed through a dedicated body channel.
/// Shared between the reader and the writer so their dispatch can never
/// disagree about which entries carry a body — `append_entry`'s
/// `RegularFile | Unknown` arm treats a mismatch as fatal rather than
/// guessing.
fn has_body(entry: &FileManifest) -> bool {
    entry.stats.is_some()
        && matches!(
            entry.file_mode(),
            FileManifestType::RegularFile | FileManifestType::Unknown
        )
}

/// Reads `entry`'s content from the pool and forwards it as chunks on
/// `body_tx`. Never returns an error itself — a read failure is relayed as a
/// message on the channel so the writer (which owns the tar stream) decides
/// what it means, exactly like a successful chunk.
async fn stream_body(
    entry: &FileManifest,
    pool_path: &Path,
    body_tx: mpsc::Sender<std::io::Result<Bytes>>,
) {
    let mut reader = Box::pin(entry.open_from_pool(pool_path));
    loop {
        let mut buf = vec![0u8; BUFFER_SIZE];
        match reader.read(&mut buf).await {
            Ok(0) => return,
            Ok(n) => {
                buf.truncate(n);
                if body_tx.send(Ok(Bytes::from(buf))).await.is_err() {
                    return; // writer already stopped reading (aborted)
                }
            }
            Err(err) => {
                let _ = body_tx.send(Err(err)).await;
                return; // this entry's body is broken; caller moves on to the next entry
            }
        }
    }
}

/// Drains `shares`' manifests in order — cheap, sequential protobuf reads,
/// never the throughput bottleneck — and round-robins each entry across
/// `dispatch_txs`, one channel per [`run_reader_worker`]. Stays a plain async
/// task (not `spawn_blocking`): no CPU-heavy work happens here, only I/O and
/// channel sends.
///
/// `tar::Builder` has no entry-ordering requirement (verified against the
/// vendored `tar` crate source: `append`/`append_data` write to the
/// underlying stream in call order with no hierarchy check — an extraction
/// convention, not a format constraint), and no test or downstream consumer
/// (`ws_console archive verify`'s checksum, in particular) depends on entry
/// order either — so round-robin dispatch, with workers reporting back to the
/// writer in whatever order they complete, is safe.
///
/// Returns `()`, not `Result`: any fatal error (a missing manifest, a broken
/// manifest entry stream) is relayed as the *last* message on `header_tx`
/// instead, so there's a single source of truth for "why did the run fail" —
/// the writer, which is the one actually driving the tar stream. On such an
/// error, or once every share has been drained, dropping `dispatch_txs` (this
/// function returning) closes every worker's dispatch channel, letting them
/// wind down once they finish whatever entry they're currently on.
async fn dispatch_entries(
    shares: Vec<(PathBuf, crate::manifest::Manifest)>,
    dispatch_txs: Vec<mpsc::Sender<(PathBuf, FileManifest)>>,
    header_tx: mpsc::Sender<Result<EntryMsg>>,
) {
    let mut next_worker = 0usize;
    for (share_prefix, manifest) in shares {
        let mut reader =
            match ProtobufReader::<FileManifest>::new(&manifest.manifest_path, true).await {
                Ok(reader) => reader,
                Err(err) => {
                    let _ = header_tx.send(Err(err.into())).await;
                    return;
                }
            };
        let mut manifest_entries = reader.into_stream();

        while let Some(entry) = manifest_entries.next().await {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    let _ = header_tx.send(Err(err.into())).await;
                    return;
                }
            };
            let archive_path = share_prefix.join(entry.path());
            let target = &dispatch_txs[next_worker % dispatch_txs.len()];
            next_worker += 1;
            if target.send((archive_path, entry)).await.is_err() {
                return; // that worker's channel is closed — writer already aborted
            }
        }
    }
}

/// One of the N parallel reader workers spawned by [`write_tar_stream`].
/// Pulls `(archive_path, entry)` pairs from its own dispatch channel
/// (populated by [`dispatch_entries`]), decompresses pool chunks, and
/// forwards completed [`EntryMsg`]s to the writer over `header_tx` — a clone
/// shared with every other worker (and the dispatcher, for fatal errors),
/// since `mpsc::Sender` supports multiple producers feeding one `Receiver`.
///
/// Runs on its own blocking-pool thread, driven by `Handle::block_on`, for
/// the same reason the single-reader design did — decompression
/// (`WoodstockCompressionReader::poll_read`) is CPU-bound work happening
/// inline in an `AsyncRead::poll_read`; running it on an async worker thread
/// would steal time from other tasks sharing the runtime.
///
/// Invariant that makes this deadlock-free, same as the single-reader design
/// generalized to N independent workers: each worker sends an entry's header
/// (and the receiving end of its body channel, if any) and then fully drains
/// and closes that body channel *before* pulling its next entry from
/// `dispatch_rx`. A writer blocked reading that body therefore always reaches
/// either data or a clean EOF. If the writer has already aborted, its
/// `header_rx` — and every `EntryMsg` (with its embedded `body_rx`) still
/// buffered in it — is dropped when its closure returns, so any worker
/// currently blocked sending on one of those now-orphaned body channels, or
/// on `header_tx` itself, fails immediately instead of blocking forever. This
/// doesn't depend on how many other workers are doing the same thing at the
/// same time.
fn run_reader_worker(
    mut dispatch_rx: mpsc::Receiver<(PathBuf, FileManifest)>,
    pool_path: PathBuf,
    header_tx: mpsc::Sender<Result<EntryMsg>>,
) {
    let handle = tokio::runtime::Handle::current();
    handle.block_on(async move {
        while let Some((archive_path, entry)) = dispatch_rx.recv().await {
            if has_body(&entry) {
                let (body_tx, body_rx) = mpsc::channel(TAR_BODY_CHANNEL_CAPACITY);
                let msg = EntryMsg {
                    archive_path,
                    entry: entry.clone(),
                    body_rx: Some(body_rx),
                };
                if header_tx.send(Ok(msg)).await.is_err() {
                    return; // writer stopped; nothing more to produce
                }
                // Header (and this channel's receiver) already handed to the
                // writer — stream the body now so the writer can start
                // draining it concurrently with this worker reading ahead,
                // and other workers decompressing their own entries in
                // parallel.
                stream_body(&entry, &pool_path, body_tx).await;
            } else {
                let msg = EntryMsg {
                    archive_path,
                    entry,
                    body_rx: None,
                };
                if header_tx.send(Ok(msg)).await.is_err() {
                    return;
                }
            }
        }
    });
}

/// Drives `tar::Builder` off entries received on `header_rx`, mirroring the
/// dispatch that used to live directly in `write_tar_stream`'s single
/// blocking closure. Runs on its own blocking-pool thread — separate from
/// `run_reader`'s — which is what lets pool reads (in `run_reader`) and
/// compression/disk writes (here) actually overlap instead of running
/// strictly one after the other.
fn run_writer(
    mut header_rx: mpsc::Receiver<Result<EntryMsg>>,
    sync_writer: SyncIoBridge<Box<dyn AsyncWrite + Send + Unpin>>,
    progress: Option<Arc<ArchiveProgressCounters>>,
    cancel_token: CancellationToken,
) -> Result<SyncIoBridge<Box<dyn AsyncWrite + Send + Unpin>>> {
    let buffered_writer = std::io::BufWriter::with_capacity(BRIDGE_BUFFER_SIZE, sync_writer);
    let mut builder = Builder::new(buffered_writer);
    builder.mode(tar::HeaderMode::Complete);

    let handle = tokio::runtime::Handle::current();

    // Dropping `header_rx` (on every `return`/`?` below, including the
    // `StreamCorrupted` abort and this cancel check) is what unblocks the
    // reader — see the invariant documented on `run_reader`.
    while let Some(msg) = handle.block_on(header_rx.recv()) {
        // `is_cancelled()` is a plain sync check, so no bridge/async
        // machinery is needed to consult it from this blocking-pool thread.
        // Returning an error here reuses `write_host_tar_archive`'s existing
        // remove-the-truncated-file cleanup path — a cancel is handled
        // exactly like a write error, not as a new code path.
        if cancel_token.is_cancelled() {
            return Err(eyre!("Archive cancelled by user"));
        }

        let EntryMsg {
            archive_path,
            entry,
            body_rx,
        } = msg?;
        match append_entry(
            &mut builder,
            &archive_path,
            &entry,
            body_rx,
            progress.as_deref(),
        ) {
            Ok(()) => {}
            Err(AppendEntryError::Skippable(err)) => {
                warn!("Skipping {:?} in archive: {err}", archive_path);
            }
            Err(AppendEntryError::ContentMissing(err)) => {
                // Louder than `Skippable`: this isn't a benign, expected
                // condition (an unsupported path, missing stats) — it means
                // this file's content is genuinely absent from the archive.
                // Nothing was written to the tar stream for it (see the peek
                // in `append_entry`), so the run can safely continue, but an
                // operator needs to notice this in their logs.
                error!(
                    "Skipping {:?} in archive (content unreadable): {err}",
                    archive_path
                );
            }
            Err(AppendEntryError::StreamCorrupted(err)) => {
                return Err(err.wrap_err(format!(
                    "tar stream corrupted while writing {archive_path:?}; aborting archive"
                )));
            }
        }
    }

    builder.finish()?;
    let buffered_writer = builder.into_inner()?;
    buffered_writer
        .into_inner()
        .map_err(|err| eyre!("failed to flush tar archive buffer: {}", err.into_error()))
}

/// `AsyncWrite` wrapper that feeds every byte actually accepted by the
/// underlying sink through a running hash. Placed *below* the compression
/// encoder (see [`write_tar_stream`]) so it hashes exactly the compressed
/// bytes landing on disk — the same bytes a post-hoc [`sha256_hex_of_file`]
/// read back would hash — letting the checksum fall out of the single write
/// pass instead of costing a second full read of the archive afterwards.
/// `hasher` is `None` when no checksum was requested, so the wrapper is a
/// transparent passthrough with one extra branch per write in that case.
struct HashingWriter<W> {
    inner: W,
    hasher: Option<Arc<StdMutex<Box<dyn ChunkHasher + Send + Sync>>>>,
}

impl<W: AsyncWrite + Unpin> AsyncWrite for HashingWriter<W> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let this = self.get_mut();
        let result = Pin::new(&mut this.inner).poll_write(cx, buf);
        if let (Poll::Ready(Ok(n)), Some(hasher)) = (&result, &this.hasher) {
            hasher.lock().unwrap().update(&buf[..*n]);
        }
        result
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_shutdown(cx)
    }
}

/// Writes the tar/compressor stream and, when the format's `checksum` option
/// is set, returns the archive's lowercase hex SHA-256 digest — computed
/// inline via [`HashingWriter`] as bytes are written, not by re-reading the
/// finished file (see [`HashingWriter`]'s docs for why that matters).
async fn write_tar_stream(
    archive_path: &Path,
    format: ArchiveFormat,
    pool_path: &Path,
    backups: &Backups,
    hostname: &str,
    backup: &Backup,
    progress: Option<Arc<ArchiveProgressCounters>>,
    cancel_token: CancellationToken,
) -> Result<Option<String>> {
    // Only the (small) list of shares is loaded up front — manifest entries
    // themselves are streamed one at a time by the reader task below, never
    // collected into memory.
    let share_paths = backups.get_backup_share_paths(hostname, backup.id).await;
    let shares: Vec<(PathBuf, crate::manifest::Manifest)> = share_paths
        .iter()
        .map(|share| {
            (
                crate::utils::path::safe_share_prefix(share),
                backups.get_manifest(hostname, backup.id, share),
            )
        })
        .collect();

    let file = File::create(archive_path).await?;
    let write_checksum = format.tar_options().is_some_and(|o| o.checksum);
    let hasher = write_checksum
        .then(|| Arc::new(StdMutex::new(create_chunk_hasher(ChunkAlgorithm::Sha2256))));
    let writer = HashingWriter {
        inner: BufWriter::new(file),
        hasher: hasher.clone(),
    };
    let level = resolve_compression_level(format);

    let boxed: Box<dyn AsyncWrite + Send + Unpin> = match format {
        ArchiveFormat::Tar(_) => Box::new(writer),
        ArchiveFormat::TarGz(_) => Box::new(GzipEncoder::with_quality(writer, level)),
        ArchiveFormat::TarXz(_) => Box::new(XzEncoder::with_quality(writer, level)),
        ArchiveFormat::TarZstd(_) => Box::new(ZstdEncoder::with_quality(writer, level)),
        ArchiveFormat::Dir => return Err(eyre!("Dir format has no tar stream")),
    };

    let sync_writer = SyncIoBridge::new(boxed);

    // Pool-chunk reads (N parallel `run_reader_worker`s, fed by
    // `dispatch_entries`) and tar/compressor writes (`run_writer`) run on
    // separate blocking-pool threads, connected by bounded channels, so
    // reading/decompressing and writing actually overlap instead of running
    // strictly one after the other — see the doc comments on
    // `dispatch_entries`/`run_reader_worker`/`run_writer` for the reasoning
    // and the anti-deadlock invariant.
    let (header_tx, header_rx) = mpsc::channel(DEFAULT_CHANNEL_BUFFER_SIZE);
    let worker_count = archive_reader_worker_count();

    let mut dispatch_txs = Vec::with_capacity(worker_count);
    let mut worker_handles = Vec::with_capacity(worker_count);
    for _ in 0..worker_count {
        let (dispatch_tx, dispatch_rx) = mpsc::channel(DISPATCH_CHANNEL_CAPACITY);
        dispatch_txs.push(dispatch_tx);
        let pool_path = pool_path.to_path_buf();
        let header_tx = header_tx.clone();
        worker_handles.push(tokio::task::spawn_blocking(move || {
            run_reader_worker(dispatch_rx, pool_path, header_tx)
        }));
    }

    let dispatch_handle = tokio::spawn(dispatch_entries(shares, dispatch_txs, header_tx.clone()));
    // The dispatcher and every worker above hold their own clone now; drop
    // this one so `header_rx` closes once all of them are done, rather than
    // staying open on this extra reference.
    drop(header_tx);

    let writer_handle = tokio::task::spawn_blocking(move || {
        run_writer(header_rx, sync_writer, progress, cancel_token)
    });

    // `tokio::join!`, not `try_join!`: always wait for EVERY task before
    // returning. `try_join!` would drop the still-running tasks' handles as
    // soon as one resolves, silently detaching them instead of waiting for
    // them to actually stop — we'd leave reader threads running orphaned in
    // the background after a writer abort.
    let (dispatch_joined, workers_joined, writer_joined) = tokio::join!(
        dispatch_handle,
        futures::future::join_all(worker_handles),
        writer_handle
    );
    dispatch_joined?; // propagate a dispatcher-task panic
    for worker_joined in workers_joined {
        worker_joined?; // propagate a reader-worker-thread panic
    }
    let sync_writer = writer_joined??; // propagate a writer-thread panic, then its Result

    let mut boxed = sync_writer.into_inner();
    boxed.shutdown().await?;

    // `shutdown` above flushed every buffered byte through `HashingWriter`,
    // so the hasher (shared via `Arc` with the writer that just finished) is
    // done being updated and safe to finalize here.
    let checksum_hex = hasher.map(|hasher| hex::encode(hasher.lock().unwrap().finalize()));

    Ok(checksum_hex)
}

/// Distinguishes `append_entry` failures that happened before any bytes were
/// written to the tar stream (safe to skip this entry and keep going) from a
/// failure that occurred after a header — and possibly part of its payload —
/// was already flushed to the archive's actual output stream. `tar::Builder`
/// writes the header immediately and copies the body straight into that same
/// stream (see `tar::Builder::append` upstream); an error mid-copy leaves the
/// stream misaligned for every entry that follows, so that case must abort
/// the whole archive run rather than continue writing into a corrupted
/// stream.
enum AppendEntryError {
    /// Benign, expected condition (missing stats, a path `tar::Builder`
    /// itself rejects) — logged quietly and skipped.
    Skippable(eyre::Report),
    /// A regular file's content could not be read at all — the pool chunk
    /// for it is missing/unreadable, detected *before* writing anything to
    /// the tar stream (see the peek in `append_entry`'s `RegularFile` arm),
    /// so it's still safe to skip and continue. Distinct from `Skippable`
    /// because it means real data loss for this one file, not a benign
    /// structural quirk — logged loudly so it doesn't go unnoticed.
    ContentMissing(eyre::Report),
    StreamCorrupted(eyre::Report),
}

/// Wraps a reader, counting the bytes actually delivered through `read`, so
/// the caller can tell a body that hit EOF early (no `Err`, just fewer bytes
/// than the manifest declared) from one that was fully copied — see the
/// length check in `append_entry`'s `RegularFile` arm.
struct CountingReader<R> {
    inner: R,
    read_total: u64,
}

impl<R> CountingReader<R> {
    fn new(inner: R) -> Self {
        Self {
            inner,
            read_total: 0,
        }
    }
}

impl<R: std::io::Read> std::io::Read for CountingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.read_total += n as u64;
        Ok(n)
    }
}

impl<R: std::io::BufRead> std::io::BufRead for CountingReader<R> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.inner.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.read_total += amt as u64;
        self.inner.consume(amt);
    }
}

fn append_entry<W: std::io::Write>(
    builder: &mut Builder<W>,
    archive_path: &Path,
    entry: &FileManifest,
    body_rx: Option<mpsc::Receiver<std::io::Result<Bytes>>>,
    progress: Option<&ArchiveProgressCounters>,
) -> Result<(), AppendEntryError> {
    let stats = entry
        .stats
        .as_ref()
        .ok_or_else(|| eyre!("manifest entry has no stats"))
        .map_err(AppendEntryError::Skippable)?;

    let mut header = Header::new_gnu();
    header.set_mode(stats.mode);
    header.set_uid(u64::from(stats.owner_id));
    header.set_gid(u64::from(stats.group_id));
    header.set_mtime(stats.last_modified.max(0).cast_unsigned());

    match entry.file_mode() {
        FileManifestType::Directory => {
            header.set_entry_type(TarEntryType::Directory);
            header.set_size(0);
            // `tar::Builder` may reject the path itself (length/encoding)
            // before writing anything — that failure leaves the stream
            // aligned, so it's safe to skip.
            builder
                .append_data(&mut header, archive_path, std::io::empty())
                .map_err(|err| AppendEntryError::Skippable(eyre!(err)))?;
            if let Some(progress) = progress {
                progress.files.fetch_add(1, Ordering::Relaxed);
            }
        }
        FileManifestType::Symlink => {
            if entry.symlink.is_empty() {
                // Known Windows scanner artifact: unresolved reparse points
                // (e.g. WindowsApps "app execution alias" stubs) are reported
                // as symlinks with no target. There is nothing meaningful to
                // link to, so skip quietly rather than warn.
                debug!("Skipping symlink with empty target: {archive_path:?}");
                return Ok(());
            }
            let target = crate::utils::path::vec_to_path(&entry.symlink);
            header.set_entry_type(TarEntryType::Symlink);
            header.set_size(0);
            // Same path-rejection reasoning as the `Directory` arm above.
            builder
                .append_link(&mut header, archive_path, &target)
                .map_err(|err| AppendEntryError::Skippable(eyre!(err)))?;
            if let Some(progress) = progress {
                progress.files.fetch_add(1, Ordering::Relaxed);
            }
        }
        FileManifestType::RegularFile | FileManifestType::Unknown => {
            let Some(body_rx) = body_rx else {
                // `has_body` (reader side) and this match (writer side) must
                // always agree on which entries carry a body — if they ever
                // disagree, abort cleanly rather than write an empty/wrong
                // entry in silence. Nothing has been written for this entry
                // yet, so aborting here is safe.
                return Err(AppendEntryError::StreamCorrupted(eyre!(
                    "no body channel for regular file entry (has_body/dispatch mismatch)"
                )));
            };
            // No `Box::pin` needed here (unlike the old direct
            // `entry.open_from_pool(...)` path): `ReceiverStream<T>` is
            // `Unpin` (it's just a channel handle), so
            // `StreamReader<ReceiverStream<_>, Bytes>` is `Unpin` too, which
            // is all `SyncIoBridge::new` requires.
            let reader = StreamReader::new(ReceiverStream::new(body_rx));
            let sync_reader = SyncIoBridge::new(reader);
            let buffered_reader =
                std::io::BufReader::with_capacity(BRIDGE_BUFFER_SIZE, sync_reader);
            let mut counting_reader = CountingReader::new(buffered_reader);

            // Peek the first read *before* writing anything to the tar
            // stream: if the pool content is entirely unreadable (a
            // missing/corrupt chunk — the common case), this fails right
            // here, nothing has touched the archive's output yet, and the
            // entry can be skipped cleanly instead of aborting the whole
            // run. `fill_buf` doesn't consume anything, so the peeked bytes
            // are still there for `append_data` below to read normally.
            if let Err(err) = std::io::BufRead::fill_buf(&mut counting_reader) {
                return Err(AppendEntryError::ContentMissing(eyre!(err)));
            }

            header.set_entry_type(TarEntryType::Regular);
            header.set_size(entry.size());
            // A failure *past* the peek — decompression breaking down
            // partway through, or a destination write error — means bytes
            // are already committed to the stream, so it must still abort.
            builder
                .append_data(&mut header, archive_path, &mut counting_reader)
                .map_err(|err| AppendEntryError::StreamCorrupted(eyre!(err)))?;

            // `append_data`'s `io::copy` stops at EOF, not at `entry.size()`
            // — a body that legitimately hits EOF early (e.g. the empty-hash
            // "corrupted chunk" case in `file_manifest_reader.rs`) succeeds
            // without ever raising an `Err`, yet leaves a header that
            // declares more bytes than the archive actually contains for
            // this entry. `pad_zeroes` pads to the *actual* copied length,
            // so a tar reader trusting the declared size will read into the
            // next entry's header — the same misalignment this whole
            // Skippable/StreamCorrupted split exists to prevent. Bytes are
            // already committed at this point, so this can only be caught
            // after the fact, and the only safe response is to abort.
            let read_total = counting_reader.read_total;
            if read_total != entry.size() {
                return Err(AppendEntryError::StreamCorrupted(eyre!(
                    "body shorter than declared size for {archive_path:?}: read {read_total} of {} bytes",
                    entry.size()
                )));
            }

            if let Some(progress) = progress {
                progress.bytes.fetch_add(entry.size(), Ordering::Relaxed);
                progress.files.fetch_add(1, Ordering::Relaxed);
            }
        }
        FileManifestType::Fifo => {
            header.set_entry_type(TarEntryType::Fifo);
            // Explicit, not `entry.size()`: a FIFO's `st_size` is unspecified by
            // POSIX. `tar::Builder::append` pads the stream to the declared size,
            // so a stray nonzero value here (with `std::io::empty()` as the body)
            // would misalign every entry written after this one.
            header.set_size(0);
            builder
                .append_data(&mut header, archive_path, std::io::empty())
                .map_err(|err| AppendEntryError::Skippable(eyre!(err)))?;
            if let Some(progress) = progress {
                progress.files.fetch_add(1, Ordering::Relaxed);
            }
        }
        FileManifestType::BlockDevice | FileManifestType::CharacterDevice => {
            header.set_entry_type(if entry.file_mode() == FileManifestType::BlockDevice {
                TarEntryType::Block
            } else {
                TarEntryType::Char
            });
            // Same reasoning as the `Fifo` arm above: a device node's `st_size` is
            // unspecified, so the size must be forced to zero rather than trusted.
            header.set_size(0);
            let (major, minor) = decode_rdev(stats.rdev);
            header
                .set_device_major(major)
                .map_err(|err| AppendEntryError::Skippable(eyre!(err)))?;
            header
                .set_device_minor(minor)
                .map_err(|err| AppendEntryError::Skippable(eyre!(err)))?;
            builder
                .append_data(&mut header, archive_path, std::io::empty())
                .map_err(|err| AppendEntryError::Skippable(eyre!(err)))?;
            if let Some(progress) = progress {
                progress.files.fetch_add(1, Ordering::Relaxed);
            }
        }
        FileManifestType::Socket => {
            // Standard tar (ustar/GNU) has no entry type for Unix domain sockets
            // — GNU tar itself skips them with a warning — so there is nothing to
            // write here. Named explicitly rather than folded into a catch-all so
            // this is a documented choice, not an accident.
            debug!("Skipping socket (unrepresentable in tar format): {archive_path:?}");
        }
    }

    Ok(())
}

/// Splits a raw `dev_t` (as returned by `MetadataExt::rdev()` on the scanning
/// client, stored verbatim in `FileManifestStat::rdev`) into the major/minor
/// pair `tar::Header::set_device_major/minor` expects.
///
/// Uses the glibc 64-bit `dev_t` encoding (`gnu_dev_major`/`gnu_dev_minor`).
/// This is correct for Linux/glibc clients; a device node captured from a
/// client with a different `dev_t` layout (e.g. FreeBSD) may decode to the
/// wrong major/minor — accepted as a known limitation since device nodes are
/// rarely present in backed-up shares.
fn decode_rdev(rdev: u64) -> (u32, u32) {
    let major = ((rdev >> 8) & 0xfff) | ((rdev >> 32) & !0xfff);
    let minor = (rdev & 0xff) | ((rdev >> 12) & !0xff);
    (major as u32, minor as u32)
}

/// Computes the lowercase hex SHA-256 digest of a file — used both to write
/// the checksum file alongside a fresh archive and by `ws_console archive
/// verify` to re-check one later.
///
/// # Errors
/// Returns an error if the file cannot be opened or read.
pub async fn sha256_hex_of_file(path: &Path) -> Result<String> {
    let mut file = File::open(path).await?;
    let mut hasher = create_chunk_hasher(ChunkAlgorithm::Sha2256);

    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    Ok(hex::encode(hasher.finalize()))
}

/// Path to the `sha256sum`-compatible checksum file alongside an archive.
#[must_use]
pub fn checksum_path_for(archive_path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.sha256", archive_path.display()))
}

async fn write_checksum_file(archive_path: &Path, hex_digest: &str) -> Result<PathBuf> {
    let file_name = archive_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| eyre!("archive path has no valid file name"))?;

    let checksum_path = checksum_path_for(archive_path);
    let content = format!("{hex_digest}  {file_name}\n");
    tokio::fs::write(&checksum_path, content).await?;

    Ok(checksum_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        BackupStatus, Backups, Configuration, ShareRecord, ShareSnapshotMethod, TarOptions,
    };
    use crate::pool::PoolChunkWrapper;
    use crate::proto::save_file;
    use crate::utils::compression::CompressionFormat;
    use crate::{FileManifestStat, FileManifestType};
    use chrono::Local;
    use futures::stream;
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

    #[tokio::test]
    async fn writes_readable_tar_gz_archive() {
        let tmp = tempfile::tempdir().unwrap();
        let config = Arc::new(Configuration::from_backup_path(tmp.path().to_path_buf()));
        let backups = Backups::new(config.clone());
        let hostname = "fakehost";
        let backup_id = Uuid::now_v7();
        let backup = fake_backup(backup_id);

        // Write one chunk into the pool.
        let file_bytes = b"hello world from woodstock archiving".to_vec();
        let mut chunk = PoolChunkWrapper::new(&config.path.pool_path, None);
        let info = chunk
            .write(
                stream::once(async { Ok(file_bytes.clone()) }),
                b"file.txt",
                ChunkAlgorithm::Blake3,
                CompressionFormat::Zstd,
            )
            .await
            .unwrap();

        // Register the share and write its manifest (one dir + one regular file).
        let dest_dir = backups.get_backup_destination_directory(hostname, backup_id);
        tokio::fs::create_dir_all(&dest_dir).await.unwrap();

        backups
            .add_backup_share_record(
                hostname,
                backup_id,
                ShareRecord {
                    path: "/share".into(),
                    snapshot_method: ShareSnapshotMethod::None,
                    snapshot_failure_reason: None,
                },
            )
            .await
            .unwrap();

        let manifest = backups.get_manifest(hostname, backup_id, "/share");
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
                size: file_bytes.len() as u64,
                ..Default::default()
            }),
            chunks: vec![info.sha256.clone()],
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

        let out_dir = tmp.path().join("out");
        let output = write_host_tar_archive(
            &backups,
            &config.path.pool_path,
            hostname,
            &backup,
            &out_dir,
            ArchiveFormat::TarGz(TarOptions {
                checksum: true,
                compression_level: None,
            }),
            None,
            CancellationToken::new(),
        )
        .await
        .unwrap();

        assert!(output.archive_path.exists());
        assert!(output.checksum_path.as_ref().unwrap().exists());

        // Prove the output is a standard, portable gzip/tar file — not the
        // Woodstock-proprietary WSCxx envelope.
        let listing = std::process::Command::new("tar")
            .arg("tzf")
            .arg(&output.archive_path)
            .output()
            .expect("failed to run system tar");
        assert!(
            listing.status.success(),
            "system tar could not read the archive: {listing:?}"
        );
        let listing = String::from_utf8_lossy(&listing.stdout);
        assert!(
            listing.contains("share/dir/file.txt"),
            "unexpected listing: {listing}"
        );

        // This is exactly what `ws_console archive verify` does: recompute
        // the archive's hash and compare it against the checksum file.
        let checksum_path = output.checksum_path.unwrap();
        assert_eq!(checksum_path, checksum_path_for(&output.archive_path));

        let checksum_file_content = tokio::fs::read_to_string(&checksum_path).await.unwrap();
        let expected_hex = checksum_file_content.split_whitespace().next().unwrap();

        let actual_hex = sha256_hex_of_file(&output.archive_path).await.unwrap();
        assert_eq!(actual_hex, expected_hex);

        // Corrupting the archive after the fact must make verification fail.
        tokio::fs::write(&output.archive_path, b"corrupted")
            .await
            .unwrap();
        let corrupted_hex = sha256_hex_of_file(&output.archive_path).await.unwrap();
        assert_ne!(corrupted_hex, expected_hex);
    }

    /// Manifest entries are now streamed one share at a time instead of being
    /// collected into a `Vec` up front — this proves that streaming didn't
    /// weaken error handling: a share whose manifest was never written must
    /// still fail the whole archive run hard, and must not leave a truncated
    /// archive file behind (the tar file is already created by the time the
    /// second share's manifest read fails, since writing starts before all
    /// shares have been walked).
    #[tokio::test]
    async fn missing_manifest_fails_archive_and_leaves_no_partial_file() {
        let tmp = tempfile::tempdir().unwrap();
        let config = Arc::new(Configuration::from_backup_path(tmp.path().to_path_buf()));
        let backups = Backups::new(config.clone());
        let hostname = "fakehost";
        let backup_id = Uuid::now_v7();
        let backup = fake_backup(backup_id);

        let file_bytes = b"hello world from woodstock archiving".to_vec();
        let mut chunk = PoolChunkWrapper::new(&config.path.pool_path, None);
        let info = chunk
            .write(
                stream::once(async { Ok(file_bytes.clone()) }),
                b"file.txt",
                ChunkAlgorithm::Blake3,
                CompressionFormat::Zstd,
            )
            .await
            .unwrap();

        let dest_dir = backups.get_backup_destination_directory(hostname, backup_id);
        tokio::fs::create_dir_all(&dest_dir).await.unwrap();

        // First share: valid manifest.
        backups
            .add_backup_share_record(
                hostname,
                backup_id,
                ShareRecord {
                    path: "/share-ok".into(),
                    snapshot_method: ShareSnapshotMethod::None,
                    snapshot_failure_reason: None,
                },
            )
            .await
            .unwrap();
        let manifest_ok = backups.get_manifest(hostname, backup_id, "/share-ok");
        let file_entry = FileManifest {
            path: b"file.txt".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::RegularFile as i32,
                mode: 0o644,
                size: file_bytes.len() as u64,
                ..Default::default()
            }),
            chunks: vec![info.sha256.clone()],
            ..Default::default()
        };
        save_file(
            &manifest_ok.manifest_path,
            stream::iter(vec![file_entry]),
            false,
            CompressionFormat::Zstd,
        )
        .await
        .unwrap();

        // Second share: registered, but its manifest file is never written.
        backups
            .add_backup_share_record(
                hostname,
                backup_id,
                ShareRecord {
                    path: "/share-missing".into(),
                    snapshot_method: ShareSnapshotMethod::None,
                    snapshot_failure_reason: None,
                },
            )
            .await
            .unwrap();

        let out_dir = tmp.path().join("out");
        let format = ArchiveFormat::TarGz(TarOptions {
            checksum: true,
            compression_level: None,
        });
        let expected_archive_path = archive_path_for(&out_dir, hostname, format).unwrap();

        let result = write_host_tar_archive(
            &backups,
            &config.path.pool_path,
            hostname,
            &backup,
            &out_dir,
            format,
            None,
            CancellationToken::new(),
        )
        .await;

        assert!(
            result.is_err(),
            "expected an error for the missing manifest"
        );
        assert!(
            !expected_archive_path.exists(),
            "a partial archive must not be left behind on failure"
        );
        assert!(!checksum_path_for(&expected_archive_path).exists());
    }

    /// A pool chunk referenced by a manifest entry can go missing (or become
    /// unreadable) independently of the manifest itself — this is a real,
    /// observed production scenario (a chunk that was never actually
    /// written to the pool during the original backup, while the manifest
    /// still recorded a hash and size for it), not just a hypothetical.
    /// `append_entry`'s peek (`std::io::BufRead::fill_buf` before writing
    /// anything to the tar stream) catches this on the very first read,
    /// before any bytes are committed — so unlike a failure *after* some
    /// bytes are already flushed (see `oversized_declared_size_still_aborts_the_run`
    /// below), this can be skipped safely: the run must continue past it and
    /// still produce every other entry, and must log it loudly
    /// (`error!`, not `warn!`) since it means real content is missing from
    /// the resulting archive.
    #[tokio::test]
    async fn missing_pool_chunk_is_skipped_and_archive_still_succeeds() {
        let tmp = tempfile::tempdir().unwrap();
        let config = Arc::new(Configuration::from_backup_path(tmp.path().to_path_buf()));
        let backups = Backups::new(config.clone());
        let hostname = "fakehost";
        let backup_id = Uuid::now_v7();
        let backup = fake_backup(backup_id);

        let dest_dir = backups.get_backup_destination_directory(hostname, backup_id);
        tokio::fs::create_dir_all(&dest_dir).await.unwrap();

        backups
            .add_backup_share_record(
                hostname,
                backup_id,
                ShareRecord {
                    path: "/share".into(),
                    snapshot_method: ShareSnapshotMethod::None,
                    snapshot_failure_reason: None,
                },
            )
            .await
            .unwrap();

        // Two real chunks, written to the pool, sandwiching the broken
        // entry — proves the run doesn't just survive a *trailing* broken
        // entry, it keeps producing entries that come after it too.
        let before_bytes = b"before the broken entry".to_vec();
        let after_bytes = b"after the broken entry, different content".to_vec();
        let mut chunk = PoolChunkWrapper::new(&config.path.pool_path, None);
        let before_info = chunk
            .write(
                stream::once({
                    let bytes = before_bytes.clone();
                    async move { Ok(bytes) }
                }),
                b"a-before.txt",
                ChunkAlgorithm::Blake3,
                CompressionFormat::Zstd,
            )
            .await
            .unwrap();
        let mut chunk = PoolChunkWrapper::new(&config.path.pool_path, None);
        let after_info = chunk
            .write(
                stream::once({
                    let bytes = after_bytes.clone();
                    async move { Ok(bytes) }
                }),
                b"c-after.txt",
                ChunkAlgorithm::Blake3,
                CompressionFormat::Zstd,
            )
            .await
            .unwrap();

        let manifest = backups.get_manifest(hostname, backup_id, "/share");
        // A single, non-empty chunk hash that was never written to the pool:
        // `open_from_pool` will fail opening it (`File::open` errors) on the
        // very first read, before any body bytes are copied — not silently
        // return zero bytes (that's the empty-chunk-hash case) or panic
        // (that's reading past the end of `chunks`). `size` is irrelevant
        // here since the failure happens before it's ever consulted.
        let before_entry = FileManifest {
            path: b"a-before.txt".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::RegularFile as i32,
                mode: 0o644,
                size: before_bytes.len() as u64,
                ..Default::default()
            }),
            chunks: vec![before_info.sha256.clone()],
            ..Default::default()
        };
        let broken_entry = FileManifest {
            path: b"b-broken.txt".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::RegularFile as i32,
                mode: 0o644,
                size: 1,
                ..Default::default()
            }),
            chunks: vec![vec![0xAAu8; 32]],
            ..Default::default()
        };
        let after_entry = FileManifest {
            path: b"c-after.txt".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::RegularFile as i32,
                mode: 0o644,
                size: after_bytes.len() as u64,
                ..Default::default()
            }),
            chunks: vec![after_info.sha256.clone()],
            ..Default::default()
        };
        save_file(
            &manifest.manifest_path,
            stream::iter(vec![before_entry, broken_entry, after_entry]),
            false,
            CompressionFormat::Zstd,
        )
        .await
        .unwrap();

        let out_dir = tmp.path().join("out");
        let format = ArchiveFormat::TarGz(TarOptions {
            checksum: true,
            compression_level: None,
        });

        let output = write_host_tar_archive(
            &backups,
            &config.path.pool_path,
            hostname,
            &backup,
            &out_dir,
            format,
            None,
            CancellationToken::new(),
        )
        .await
        .unwrap();

        assert!(output.archive_path.exists());
        assert!(output.checksum_path.unwrap().exists());

        // Extract with the system `tar` (not this crate's `tar::Archive`) so
        // a real, independent implementation confirms the archive isn't
        // just internally self-consistent but actually well-formed.
        let extract_dir = tmp.path().join("extracted");
        std::fs::create_dir_all(&extract_dir).unwrap();
        let status = std::process::Command::new("tar")
            .arg("xzf")
            .arg(&output.archive_path)
            .arg("-C")
            .arg(&extract_dir)
            .status()
            .expect("failed to run system tar");
        assert!(status.success(), "system tar failed to extract the archive");

        assert_eq!(
            std::fs::read(extract_dir.join("share/a-before.txt")).unwrap(),
            before_bytes
        );
        assert_eq!(
            std::fs::read(extract_dir.join("share/c-after.txt")).unwrap(),
            after_bytes
        );
        assert!(
            !extract_dir.join("share/b-broken.txt").exists(),
            "the entry with the missing chunk must be skipped, not present with garbage content"
        );
    }

    /// The counterpart to the missing-chunk case above: `append_data`'s
    /// `io::copy` stops at EOF, not at the header's declared size, so a body
    /// that legitimately (successfully) produces *fewer* bytes than
    /// `entry.size()` claims doesn't fail at all from `append_data`'s point
    /// of view — bytes are already committed to the tar stream by the time
    /// `append_entry` notices the mismatch. Unlike the missing-chunk case,
    /// this can't be caught by peeking the first read (the first read
    /// succeeds fine here) and can't be safely skipped — the header already
    /// lies about this entry's size, which misaligns every entry after it
    /// for a reader that trusts the declared size. Reproduced directly by
    /// writing a real, valid, short chunk but recording a much larger
    /// `stats.size` in the manifest — deterministic, no reliance on
    /// decompression-internals timing.
    #[tokio::test]
    async fn oversized_declared_size_still_aborts_the_run() {
        let tmp = tempfile::tempdir().unwrap();
        let config = Arc::new(Configuration::from_backup_path(tmp.path().to_path_buf()));
        let backups = Backups::new(config.clone());
        let hostname = "fakehost";
        let backup_id = Uuid::now_v7();
        let backup = fake_backup(backup_id);

        let dest_dir = backups.get_backup_destination_directory(hostname, backup_id);
        tokio::fs::create_dir_all(&dest_dir).await.unwrap();
        backups
            .add_backup_share_record(
                hostname,
                backup_id,
                ShareRecord {
                    path: "/share".into(),
                    snapshot_method: ShareSnapshotMethod::None,
                    snapshot_failure_reason: None,
                },
            )
            .await
            .unwrap();

        let short_bytes = b"only fifty bytes or so of real content here".to_vec();
        let mut chunk = PoolChunkWrapper::new(&config.path.pool_path, None);
        let info = chunk
            .write(
                stream::once({
                    let bytes = short_bytes.clone();
                    async move { Ok(bytes) }
                }),
                b"broken.txt",
                ChunkAlgorithm::Blake3,
                CompressionFormat::Zstd,
            )
            .await
            .unwrap();

        let manifest = backups.get_manifest(hostname, backup_id, "/share");
        let broken_entry = FileManifest {
            path: b"broken.txt".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::RegularFile as i32,
                mode: 0o644,
                // Declares far more than the chunk actually contains.
                size: short_bytes.len() as u64 + 10_000,
                ..Default::default()
            }),
            chunks: vec![info.sha256.clone()],
            ..Default::default()
        };
        save_file(
            &manifest.manifest_path,
            stream::iter(vec![broken_entry]),
            false,
            CompressionFormat::Zstd,
        )
        .await
        .unwrap();

        let out_dir = tmp.path().join("out");
        let format = ArchiveFormat::TarGz(TarOptions {
            checksum: true,
            compression_level: None,
        });
        let expected_archive_path = archive_path_for(&out_dir, hostname, format).unwrap();

        let result = write_host_tar_archive(
            &backups,
            &config.path.pool_path,
            hostname,
            &backup,
            &out_dir,
            format,
            None,
            CancellationToken::new(),
        )
        .await;

        assert!(
            result.is_err(),
            "expected an error for the declared-size mismatch"
        );
        assert!(
            !expected_archive_path.exists(),
            "a corrupted/partial archive must not be left behind on failure"
        );
        assert!(!checksum_path_for(&expected_archive_path).exists());
    }

    /// Reproduces the two real-world Windows-host issues seen in production:
    /// a raw `C:\` share collapsing into an untarrable single path component,
    /// and a WindowsApps "app execution alias" reported as a symlink with an
    /// empty target (which `tar::Builder::append_link` rejects outright).
    #[tokio::test]
    async fn writes_readable_tar_gz_archive_with_windows_share_and_empty_symlink() {
        let tmp = tempfile::tempdir().unwrap();
        let config = Arc::new(Configuration::from_backup_path(tmp.path().to_path_buf()));
        let backups = Backups::new(config.clone());
        let hostname = "windowshost";
        let backup_id = Uuid::now_v7();
        let backup = fake_backup(backup_id);

        let file_bytes = b"hello from a windows share".to_vec();
        let mut chunk = PoolChunkWrapper::new(&config.path.pool_path, None);
        let info = chunk
            .write(
                stream::once(async { Ok(file_bytes.clone()) }),
                b"file.txt",
                ChunkAlgorithm::Blake3,
                CompressionFormat::Zstd,
            )
            .await
            .unwrap();

        let dest_dir = backups.get_backup_destination_directory(hostname, backup_id);
        tokio::fs::create_dir_all(&dest_dir).await.unwrap();

        // Raw Windows share identifier, exactly as stored in `hosts/<name>.yml`.
        let share = "C:\\";
        backups
            .add_backup_share_record(
                hostname,
                backup_id,
                ShareRecord {
                    path: share.into(),
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
                size: file_bytes.len() as u64,
                ..Default::default()
            }),
            chunks: vec![info.sha256.clone()],
            ..Default::default()
        };
        // A WindowsApps "app execution alias" stub: reported as a symlink,
        // but with no resolvable target.
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

        let out_dir = tmp.path().join("out");
        let progress = Arc::new(ArchiveProgressCounters::default());
        let output = write_host_tar_archive(
            &backups,
            &config.path.pool_path,
            hostname,
            &backup,
            &out_dir,
            ArchiveFormat::TarGz(TarOptions {
                checksum: false,
                compression_level: None,
            }),
            Some(progress.clone()),
            CancellationToken::new(),
        )
        .await
        .unwrap();

        let listing = std::process::Command::new("tar")
            .arg("tzf")
            .arg(&output.archive_path)
            .output()
            .expect("failed to run system tar");
        assert!(
            listing.status.success(),
            "system tar could not read the archive: {listing:?}"
        );
        let listing = String::from_utf8_lossy(&listing.stdout);

        // The share normalizes to `C/...`, not the untarrable `C:\/...`.
        assert!(
            listing.contains("C/dir/file.txt"),
            "unexpected listing: {listing}"
        );
        // The empty-target symlink is skipped, not present under any path.
        assert!(
            !listing.contains("python3.exe"),
            "broken symlink should have been skipped: {listing}"
        );

        // The progress counter only counts the regular file — the skipped
        // (empty-target) symlink contributes nothing to either counter.
        let (bytes, files) = progress.snapshot();
        assert_eq!(bytes, file_bytes.len() as u64);
        assert_eq!(files, 1);
    }

    #[test]
    fn decode_rdev_splits_glibc_encoded_major_minor() {
        // `/dev/sda1`-style device numbers (major 8, minor 1), encoded with the
        // glibc 64-bit `dev_t` layout `decode_rdev` is meant to invert.
        let rdev = (8u64 << 8) | 1u64;
        assert_eq!(decode_rdev(rdev), (8, 1));
    }

    /// Proves `Fifo`, `BlockDevice` and `CharacterDevice` entries are written
    /// as real tar special-file headers (not silently dropped by the old
    /// catch-all), with the device entries carrying the major/minor decoded
    /// from `stats.rdev` — and that `Socket` stays deliberately absent, since
    /// standard tar has no entry type for it. Reads back with `tar::Archive`
    /// (not the system `tar` binary) because entry type / device numbers
    /// aren't visible in a plain `tzf` listing.
    #[tokio::test]
    async fn writes_fifo_and_device_entries_and_skips_socket() {
        let tmp = tempfile::tempdir().unwrap();
        let config = Arc::new(Configuration::from_backup_path(tmp.path().to_path_buf()));
        let backups = Backups::new(config.clone());
        let hostname = "fakehost";
        let backup_id = Uuid::now_v7();
        let backup = fake_backup(backup_id);

        let dest_dir = backups.get_backup_destination_directory(hostname, backup_id);
        tokio::fs::create_dir_all(&dest_dir).await.unwrap();
        backups
            .add_backup_share_record(
                hostname,
                backup_id,
                ShareRecord {
                    path: "/share".into(),
                    snapshot_method: ShareSnapshotMethod::None,
                    snapshot_failure_reason: None,
                },
            )
            .await
            .unwrap();

        // Major 8 / minor 1, glibc-encoded — matches typical `/dev/sda1` device
        // numbers, exactly the pair `decode_rdev` above proves round-trips.
        let rdev = (8u64 << 8) | 1u64;

        let fifo_entry = FileManifest {
            path: b"fifo".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::Fifo as i32,
                mode: 0o644,
                ..Default::default()
            }),
            ..Default::default()
        };
        let block_entry = FileManifest {
            path: b"block".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::BlockDevice as i32,
                mode: 0o660,
                rdev,
                ..Default::default()
            }),
            ..Default::default()
        };
        let char_entry = FileManifest {
            path: b"char".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::CharacterDevice as i32,
                mode: 0o660,
                rdev,
                ..Default::default()
            }),
            ..Default::default()
        };
        let socket_entry = FileManifest {
            path: b"socket".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::Socket as i32,
                mode: 0o755,
                ..Default::default()
            }),
            ..Default::default()
        };

        let manifest = backups.get_manifest(hostname, backup_id, "/share");
        save_file(
            &manifest.manifest_path,
            stream::iter(vec![fifo_entry, block_entry, char_entry, socket_entry]),
            false,
            CompressionFormat::Zstd,
        )
        .await
        .unwrap();

        let out_dir = tmp.path().join("out");
        let output = write_host_tar_archive(
            &backups,
            &config.path.pool_path,
            hostname,
            &backup,
            &out_dir,
            ArchiveFormat::Tar(TarOptions::default()),
            None,
            CancellationToken::new(),
        )
        .await
        .unwrap();

        let file = std::fs::File::open(&output.archive_path).unwrap();
        let mut archive = tar::Archive::new(file);
        let mut seen = std::collections::HashMap::new();
        for entry in archive.entries().unwrap() {
            let entry = entry.unwrap();
            let header = entry.header().clone();
            let path = entry.path().unwrap().to_path_buf();
            seen.insert(path, header);
        }

        let fifo_header = seen
            .get(std::path::Path::new("share/fifo"))
            .expect("fifo entry missing from archive");
        assert_eq!(fifo_header.entry_type(), tar::EntryType::Fifo);

        let block_header = seen
            .get(std::path::Path::new("share/block"))
            .expect("block device entry missing from archive");
        assert_eq!(block_header.entry_type(), tar::EntryType::Block);
        assert_eq!(block_header.device_major().unwrap(), Some(8));
        assert_eq!(block_header.device_minor().unwrap(), Some(1));

        let char_header = seen
            .get(std::path::Path::new("share/char"))
            .expect("character device entry missing from archive");
        assert_eq!(char_header.entry_type(), tar::EntryType::Char);
        assert_eq!(char_header.device_major().unwrap(), Some(8));
        assert_eq!(char_header.device_minor().unwrap(), Some(1));

        assert!(
            !seen.contains_key(std::path::Path::new("share/socket")),
            "socket entry should have been skipped: tar has no representation for it"
        );
    }

    fn precise_level(level: Level) -> i32 {
        match level {
            Level::Precise(quality) => quality,
            other => panic!("expected Level::Precise, got {other:?}"),
        }
    }

    #[test]
    fn resolve_compression_level_uses_per_format_default_when_unset() {
        assert_eq!(
            precise_level(resolve_compression_level(ArchiveFormat::TarGz(
                TarOptions::default()
            ))),
            6
        );
        assert_eq!(
            precise_level(resolve_compression_level(ArchiveFormat::TarXz(
                TarOptions::default()
            ))),
            6
        );
        assert_eq!(
            precise_level(resolve_compression_level(ArchiveFormat::TarZstd(
                TarOptions::default()
            ))),
            3
        );
    }

    #[test]
    fn resolve_compression_level_uses_requested_value_verbatim() {
        // Clamping to each codec's valid range happens downstream, inside the
        // codec's own `From<Level>` impl (see `resolve_compression_level`'s
        // doc comment) — this just checks the requested value is threaded
        // through rather than silently replaced by the default.
        assert_eq!(
            precise_level(resolve_compression_level(ArchiveFormat::TarGz(
                TarOptions {
                    checksum: false,
                    compression_level: Some(9),
                }
            ))),
            9
        );
        assert_eq!(
            precise_level(resolve_compression_level(ArchiveFormat::TarZstd(
                TarOptions {
                    checksum: false,
                    compression_level: Some(19),
                }
            ))),
            19
        );
    }

    #[tokio::test]
    async fn out_of_range_compression_level_does_not_fail_the_archive_run() {
        let tmp = tempfile::tempdir().unwrap();
        let config = Arc::new(Configuration::from_backup_path(tmp.path().to_path_buf()));
        let backups = Backups::new(config.clone());
        let hostname = "fakehost";
        let backup_id = Uuid::now_v7();
        let backup = fake_backup(backup_id);

        let file_bytes = b"hello world from woodstock archiving".to_vec();
        let mut chunk = PoolChunkWrapper::new(&config.path.pool_path, None);
        let info = chunk
            .write(
                stream::once(async { Ok(file_bytes.clone()) }),
                b"file.txt",
                ChunkAlgorithm::Blake3,
                CompressionFormat::Zstd,
            )
            .await
            .unwrap();

        let dest_dir = backups.get_backup_destination_directory(hostname, backup_id);
        tokio::fs::create_dir_all(&dest_dir).await.unwrap();
        backups
            .add_backup_share_record(
                hostname,
                backup_id,
                ShareRecord {
                    path: "/share".into(),
                    snapshot_method: ShareSnapshotMethod::None,
                    snapshot_failure_reason: None,
                },
            )
            .await
            .unwrap();

        let manifest = backups.get_manifest(hostname, backup_id, "/share");
        let file_entry = FileManifest {
            path: b"file.txt".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::RegularFile as i32,
                mode: 0o644,
                size: file_bytes.len() as u64,
                ..Default::default()
            }),
            chunks: vec![info.sha256.clone()],
            ..Default::default()
        };
        save_file(
            &manifest.manifest_path,
            stream::iter(vec![file_entry]),
            false,
            CompressionFormat::Zstd,
        )
        .await
        .unwrap();

        let out_dir = tmp.path().join("out");
        // A YAML typo (`compressionLevel: 99`) must degrade gracefully — via
        // the codec's own clamping — rather than fail a scheduled run.
        let output = write_host_tar_archive(
            &backups,
            &config.path.pool_path,
            hostname,
            &backup,
            &out_dir,
            ArchiveFormat::TarGz(TarOptions {
                checksum: false,
                compression_level: Some(99),
            }),
            None,
            CancellationToken::new(),
        )
        .await
        .unwrap();

        assert!(output.archive_path.exists());
    }

    /// Proves `TarZstd` writes a standard `.tar.zst` archive, readable by the
    /// system `tar --zstd` (not `WoodstockCompressionWriter`'s proprietary
    /// envelope), matching the tar.gz coverage above.
    #[tokio::test]
    async fn writes_readable_tar_zstd_archive() {
        let tmp = tempfile::tempdir().unwrap();
        let config = Arc::new(Configuration::from_backup_path(tmp.path().to_path_buf()));
        let backups = Backups::new(config.clone());
        let hostname = "fakehost";
        let backup_id = Uuid::now_v7();
        let backup = fake_backup(backup_id);

        let file_bytes = b"hello world from woodstock archiving".to_vec();
        let mut chunk = PoolChunkWrapper::new(&config.path.pool_path, None);
        let info = chunk
            .write(
                stream::once(async { Ok(file_bytes.clone()) }),
                b"file.txt",
                ChunkAlgorithm::Blake3,
                CompressionFormat::Zstd,
            )
            .await
            .unwrap();

        let dest_dir = backups.get_backup_destination_directory(hostname, backup_id);
        tokio::fs::create_dir_all(&dest_dir).await.unwrap();
        backups
            .add_backup_share_record(
                hostname,
                backup_id,
                ShareRecord {
                    path: "/share".into(),
                    snapshot_method: ShareSnapshotMethod::None,
                    snapshot_failure_reason: None,
                },
            )
            .await
            .unwrap();

        let manifest = backups.get_manifest(hostname, backup_id, "/share");
        let file_entry = FileManifest {
            path: b"dir/file.txt".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::RegularFile as i32,
                mode: 0o644,
                size: file_bytes.len() as u64,
                ..Default::default()
            }),
            chunks: vec![info.sha256.clone()],
            ..Default::default()
        };
        save_file(
            &manifest.manifest_path,
            stream::iter(vec![file_entry]),
            false,
            CompressionFormat::Zstd,
        )
        .await
        .unwrap();

        let out_dir = tmp.path().join("out");
        let output = write_host_tar_archive(
            &backups,
            &config.path.pool_path,
            hostname,
            &backup,
            &out_dir,
            ArchiveFormat::TarZstd(TarOptions {
                checksum: true,
                compression_level: None,
            }),
            None,
            CancellationToken::new(),
        )
        .await
        .unwrap();

        assert!(output.archive_path.ends_with(format!("{hostname}.tar.zst")));

        let listing = std::process::Command::new("tar")
            .arg("--zstd")
            .arg("-tf")
            .arg(&output.archive_path)
            .output()
            .expect("failed to run system tar");
        assert!(
            listing.status.success(),
            "system tar --zstd could not read the archive: {listing:?}"
        );
        let listing = String::from_utf8_lossy(&listing.stdout);
        assert!(
            listing.contains("share/dir/file.txt"),
            "unexpected listing: {listing}"
        );

        let checksum_path = output.checksum_path.unwrap();
        let checksum_file_content = tokio::fs::read_to_string(&checksum_path).await.unwrap();
        let expected_hex = checksum_file_content.split_whitespace().next().unwrap();
        let actual_hex = sha256_hex_of_file(&output.archive_path).await.unwrap();
        assert_eq!(actual_hex, expected_hex);
    }

    /// Proves `TarXz` (switched from `XzEncoder::new` to `::with_quality`
    /// alongside the other two formats) still writes a readable `.tar.xz`.
    #[tokio::test]
    async fn writes_readable_tar_xz_archive() {
        let tmp = tempfile::tempdir().unwrap();
        let config = Arc::new(Configuration::from_backup_path(tmp.path().to_path_buf()));
        let backups = Backups::new(config.clone());
        let hostname = "fakehost";
        let backup_id = Uuid::now_v7();
        let backup = fake_backup(backup_id);

        let file_bytes = b"hello world from woodstock archiving".to_vec();
        let mut chunk = PoolChunkWrapper::new(&config.path.pool_path, None);
        let info = chunk
            .write(
                stream::once(async { Ok(file_bytes.clone()) }),
                b"file.txt",
                ChunkAlgorithm::Blake3,
                CompressionFormat::Zstd,
            )
            .await
            .unwrap();

        let dest_dir = backups.get_backup_destination_directory(hostname, backup_id);
        tokio::fs::create_dir_all(&dest_dir).await.unwrap();
        backups
            .add_backup_share_record(
                hostname,
                backup_id,
                ShareRecord {
                    path: "/share".into(),
                    snapshot_method: ShareSnapshotMethod::None,
                    snapshot_failure_reason: None,
                },
            )
            .await
            .unwrap();

        let manifest = backups.get_manifest(hostname, backup_id, "/share");
        let file_entry = FileManifest {
            path: b"dir/file.txt".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::RegularFile as i32,
                mode: 0o644,
                size: file_bytes.len() as u64,
                ..Default::default()
            }),
            chunks: vec![info.sha256.clone()],
            ..Default::default()
        };
        save_file(
            &manifest.manifest_path,
            stream::iter(vec![file_entry]),
            false,
            CompressionFormat::Zstd,
        )
        .await
        .unwrap();

        let out_dir = tmp.path().join("out");
        let output = write_host_tar_archive(
            &backups,
            &config.path.pool_path,
            hostname,
            &backup,
            &out_dir,
            ArchiveFormat::TarXz(TarOptions {
                checksum: false,
                compression_level: None,
            }),
            None,
            CancellationToken::new(),
        )
        .await
        .unwrap();

        assert!(output.archive_path.ends_with(format!("{hostname}.tar.xz")));

        let listing = std::process::Command::new("tar")
            .arg("-Jtf")
            .arg(&output.archive_path)
            .output()
            .expect("failed to run system tar");
        assert!(
            listing.status.success(),
            "system tar -J could not read the archive: {listing:?}"
        );
        let listing = String::from_utf8_lossy(&listing.stdout);
        assert!(
            listing.contains("share/dir/file.txt"),
            "unexpected listing: {listing}"
        );
    }

    /// Proves the reader/writer pipeline's framing is correct across several
    /// entries, including one spanning multiple `BUFFER_SIZE` (128KB) reads —
    /// a single-buffer file can't distinguish correct framing from a broken
    /// one (e.g. the `.take(entry.size())`-on-a-shared-channel design this
    /// pipeline deliberately avoids, which would silently borrow bytes across
    /// entry boundaries). Re-reads the archive with `tar::Archive` (not the
    /// system `tar` binary) so each header's declared size can be compared
    /// against the number of body bytes actually read back.
    ///
    /// Runs on a genuine multi-thread runtime (not the default current-thread
    /// `#[tokio::test]`) — `job_worker` runs `#[tokio::main]`, which is
    /// multi-thread by default, and this is the flavor under which the
    /// reader and writer `spawn_blocking` threads can actually run
    /// concurrently rather than being serialized onto a single scheduler.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pipeline_preserves_multi_file_framing() {
        let tmp = tempfile::tempdir().unwrap();
        let config = Arc::new(Configuration::from_backup_path(tmp.path().to_path_buf()));
        let backups = Backups::new(config.clone());
        let hostname = "fakehost";
        let backup_id = Uuid::now_v7();
        let backup = fake_backup(backup_id);

        let dest_dir = backups.get_backup_destination_directory(hostname, backup_id);
        tokio::fs::create_dir_all(&dest_dir).await.unwrap();
        backups
            .add_backup_share_record(
                hostname,
                backup_id,
                ShareRecord {
                    path: "/share".into(),
                    snapshot_method: ShareSnapshotMethod::None,
                    snapshot_failure_reason: None,
                },
            )
            .await
            .unwrap();

        // A file spanning several `BUFFER_SIZE` (128KB) reads, sandwiched
        // between two small files — this is what would catch a framing bug
        // that a single-buffer fixture couldn't.
        let small_a = b"first small file".to_vec();
        let big = vec![0x5Au8; BUFFER_SIZE * 3 + 12_345];
        let small_b = b"second small file, different content".to_vec();

        let mut pool_entries = Vec::new();
        for (name, bytes) in [("a.txt", &small_a), ("big.bin", &big), ("b.txt", &small_b)] {
            let mut chunk = PoolChunkWrapper::new(&config.path.pool_path, None);
            let info = chunk
                .write(
                    stream::once({
                        let bytes = bytes.clone();
                        async move { Ok(bytes) }
                    }),
                    name.as_bytes(),
                    ChunkAlgorithm::Blake3,
                    CompressionFormat::Zstd,
                )
                .await
                .unwrap();
            pool_entries.push((name, bytes.clone(), info));
        }

        let manifest = backups.get_manifest(hostname, backup_id, "/share");
        let file_entries: Vec<FileManifest> = pool_entries
            .iter()
            .map(|(name, bytes, info)| FileManifest {
                path: name.as_bytes().to_vec(),
                stats: Some(FileManifestStat {
                    file_type: FileManifestType::RegularFile as i32,
                    mode: 0o644,
                    size: bytes.len() as u64,
                    ..Default::default()
                }),
                chunks: vec![info.sha256.clone()],
                ..Default::default()
            })
            .collect();
        save_file(
            &manifest.manifest_path,
            stream::iter(file_entries),
            false,
            CompressionFormat::Zstd,
        )
        .await
        .unwrap();

        let out_dir = tmp.path().join("out");
        let output = write_host_tar_archive(
            &backups,
            &config.path.pool_path,
            hostname,
            &backup,
            &out_dir,
            ArchiveFormat::Tar(TarOptions::default()),
            None,
            CancellationToken::new(),
        )
        .await
        .unwrap();

        let file = std::fs::File::open(&output.archive_path).unwrap();
        let mut archive = tar::Archive::new(file);
        let mut seen = std::collections::HashMap::new();
        for entry in archive.entries().unwrap() {
            let mut entry = entry.unwrap();
            let declared_size = entry.header().size().unwrap();
            let path = entry.path().unwrap().to_path_buf();
            let mut body = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut body).unwrap();
            assert_eq!(
                declared_size,
                body.len() as u64,
                "header size vs. actual body length mismatch for {path:?}"
            );
            seen.insert(path, body);
        }

        assert_eq!(
            seen.get(std::path::Path::new("share/a.txt")).unwrap(),
            &small_a
        );
        assert_eq!(
            seen.get(std::path::Path::new("share/big.bin")).unwrap(),
            &big
        );
        assert_eq!(
            seen.get(std::path::Path::new("share/b.txt")).unwrap(),
            &small_b
        );
    }

    /// An empty regular file (`size: 0`, no chunks) never has any bytes to
    /// stream — its body channel opens and closes without a single message.
    /// Proves that path stays correct without a special case in the code.
    #[tokio::test]
    async fn pipeline_handles_empty_regular_file() {
        let tmp = tempfile::tempdir().unwrap();
        let config = Arc::new(Configuration::from_backup_path(tmp.path().to_path_buf()));
        let backups = Backups::new(config.clone());
        let hostname = "fakehost";
        let backup_id = Uuid::now_v7();
        let backup = fake_backup(backup_id);

        let dest_dir = backups.get_backup_destination_directory(hostname, backup_id);
        tokio::fs::create_dir_all(&dest_dir).await.unwrap();
        backups
            .add_backup_share_record(
                hostname,
                backup_id,
                ShareRecord {
                    path: "/share".into(),
                    snapshot_method: ShareSnapshotMethod::None,
                    snapshot_failure_reason: None,
                },
            )
            .await
            .unwrap();

        let manifest = backups.get_manifest(hostname, backup_id, "/share");
        let empty_entry = FileManifest {
            path: b"empty.txt".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::RegularFile as i32,
                mode: 0o644,
                size: 0,
                ..Default::default()
            }),
            chunks: vec![],
            ..Default::default()
        };
        save_file(
            &manifest.manifest_path,
            stream::iter(vec![empty_entry]),
            false,
            CompressionFormat::Zstd,
        )
        .await
        .unwrap();

        let out_dir = tmp.path().join("out");
        let output = write_host_tar_archive(
            &backups,
            &config.path.pool_path,
            hostname,
            &backup,
            &out_dir,
            ArchiveFormat::Tar(TarOptions::default()),
            None,
            CancellationToken::new(),
        )
        .await
        .unwrap();

        let file = std::fs::File::open(&output.archive_path).unwrap();
        let mut archive = tar::Archive::new(file);
        let mut entries = archive.entries().unwrap();
        let mut entry = entries.next().unwrap().unwrap();
        assert_eq!(entry.header().size().unwrap(), 0);
        let mut body = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut body).unwrap();
        assert!(body.is_empty());
        assert!(entries.next().is_none());
    }

    /// The reader/writer pipeline must never deadlock when the writer aborts
    /// partway through: once it sees `AppendEntryError::StreamCorrupted` it
    /// drops its `header_rx`, which must unblock the reader (still trying to
    /// send many more entries) rather than leaving it stuck forever. Wrapped
    /// in a timeout so a regression fails fast instead of hanging the test
    /// suite. Multi-thread runtime for the same reason as
    /// `pipeline_preserves_multi_file_framing` above — this is the flavor
    /// (matching `job_worker`'s `#[tokio::main]`) under which the reader and
    /// writer threads genuinely run at the same time instead of being
    /// serialized onto one scheduler, which is exactly what could hide a
    /// deadlock that only shows up under real concurrency.
    ///
    /// Uses the declared-size-mismatch fixture (not a missing chunk) to
    /// trigger `StreamCorrupted`, since a missing chunk is now caught by
    /// `append_entry`'s peek and only ever produces `ContentMissing` — see
    /// `pipeline_skips_many_broken_entries_without_hanging` below for the
    /// equivalent liveness proof on that (non-aborting) path.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pipeline_aborts_promptly_after_stream_corruption() {
        let tmp = tempfile::tempdir().unwrap();
        let config = Arc::new(Configuration::from_backup_path(tmp.path().to_path_buf()));
        let backups = Backups::new(config.clone());
        let hostname = "fakehost";
        let backup_id = Uuid::now_v7();
        let backup = fake_backup(backup_id);

        let dest_dir = backups.get_backup_destination_directory(hostname, backup_id);
        tokio::fs::create_dir_all(&dest_dir).await.unwrap();
        backups
            .add_backup_share_record(
                hostname,
                backup_id,
                ShareRecord {
                    path: "/share".into(),
                    snapshot_method: ShareSnapshotMethod::None,
                    snapshot_failure_reason: None,
                },
            )
            .await
            .unwrap();

        let short_bytes = b"short content, way under the declared size".to_vec();
        let mut chunk = PoolChunkWrapper::new(&config.path.pool_path, None);
        let info = chunk
            .write(
                stream::once({
                    let bytes = short_bytes.clone();
                    async move { Ok(bytes) }
                }),
                b"broken.txt",
                ChunkAlgorithm::Blake3,
                CompressionFormat::Zstd,
            )
            .await
            .unwrap();

        let manifest = backups.get_manifest(hostname, backup_id, "/share");
        let mut entries = vec![FileManifest {
            path: b"broken.txt".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::RegularFile as i32,
                mode: 0o644,
                size: short_bytes.len() as u64 + 10_000, // declares far more than is actually there
                ..Default::default()
            }),
            chunks: vec![info.sha256.clone()],
            ..Default::default()
        }];
        // Many entries after the broken one — the reader producing them must
        // not get stuck trying to hand them to a writer that already
        // aborted and stopped reading.
        for i in 0..50 {
            entries.push(FileManifest {
                path: format!("after-{i}.txt").into_bytes(),
                stats: Some(FileManifestStat {
                    file_type: FileManifestType::Directory as i32,
                    mode: 0o755,
                    ..Default::default()
                }),
                ..Default::default()
            });
        }
        save_file(
            &manifest.manifest_path,
            stream::iter(entries),
            false,
            CompressionFormat::Zstd,
        )
        .await
        .unwrap();

        let out_dir = tmp.path().join("out");
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            write_host_tar_archive(
                &backups,
                &config.path.pool_path,
                hostname,
                &backup,
                &out_dir,
                ArchiveFormat::Tar(TarOptions::default()),
                None,
                CancellationToken::new(),
            ),
        )
        .await
        .expect("archive run deadlocked instead of aborting");

        assert!(
            result.is_err(),
            "expected an error for the declared-size mismatch"
        );
    }

    /// Liveness counterpart for the *non*-aborting path: many broken
    /// (missing-chunk) entries interleaved with good ones must all be
    /// skipped-and-continued without the pipeline ever stalling — the
    /// reader keeps opening a fresh body channel per entry and the writer
    /// keeps draining `header_rx` in the same steady rhythm regardless of
    /// how many entries in a row turn out to be `ContentMissing`. Wrapped in
    /// a timeout for the same reason as the abort case above.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pipeline_skips_many_broken_entries_without_hanging() {
        let tmp = tempfile::tempdir().unwrap();
        let config = Arc::new(Configuration::from_backup_path(tmp.path().to_path_buf()));
        let backups = Backups::new(config.clone());
        let hostname = "fakehost";
        let backup_id = Uuid::now_v7();
        let backup = fake_backup(backup_id);

        let dest_dir = backups.get_backup_destination_directory(hostname, backup_id);
        tokio::fs::create_dir_all(&dest_dir).await.unwrap();
        backups
            .add_backup_share_record(
                hostname,
                backup_id,
                ShareRecord {
                    path: "/share".into(),
                    snapshot_method: ShareSnapshotMethod::None,
                    snapshot_failure_reason: None,
                },
            )
            .await
            .unwrap();

        let manifest = backups.get_manifest(hostname, backup_id, "/share");
        let mut entries = Vec::new();
        for i in 0..50 {
            entries.push(FileManifest {
                path: format!("broken-{i}.txt").into_bytes(),
                stats: Some(FileManifestStat {
                    file_type: FileManifestType::RegularFile as i32,
                    mode: 0o644,
                    size: 1,
                    ..Default::default()
                }),
                chunks: vec![vec![0xAAu8; 32]], // never written to the pool
                ..Default::default()
            });
        }
        save_file(
            &manifest.manifest_path,
            stream::iter(entries),
            false,
            CompressionFormat::Zstd,
        )
        .await
        .unwrap();

        let out_dir = tmp.path().join("out");
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            write_host_tar_archive(
                &backups,
                &config.path.pool_path,
                hostname,
                &backup,
                &out_dir,
                ArchiveFormat::Tar(TarOptions::default()),
                None,
                CancellationToken::new(),
            ),
        )
        .await
        .expect("archive run hung instead of skipping the broken entries");

        assert!(
            result.is_ok(),
            "an archive with only unreadable files should still succeed (just be near-empty)"
        );
    }

    /// Same intent as `pipeline_preserves_multi_file_framing`, but with
    /// enough files (well beyond `archive_reader_worker_count()`'s default of
    /// 4) that
    /// several reader workers are genuinely decompressing different files at
    /// the same time, completing and handing entries to the writer in
    /// whatever order they finish — not necessarily manifest order. Proves
    /// that round-robin dispatch + completion-order writing never mixes up
    /// which body belongs to which header: every entry's declared tar header
    /// size must match its actual body length and content, exactly as with a
    /// single reader.
    #[tokio::test]
    async fn pipeline_preserves_framing_with_many_files_across_reader_workers() {
        let tmp = tempfile::tempdir().unwrap();
        let config = Arc::new(Configuration::from_backup_path(tmp.path().to_path_buf()));
        let backups = Backups::new(config.clone());
        let hostname = "fakehost";
        let backup_id = Uuid::now_v7();
        let backup = fake_backup(backup_id);

        let dest_dir = backups.get_backup_destination_directory(hostname, backup_id);
        tokio::fs::create_dir_all(&dest_dir).await.unwrap();
        backups
            .add_backup_share_record(
                hostname,
                backup_id,
                ShareRecord {
                    path: "/share".into(),
                    snapshot_method: ShareSnapshotMethod::None,
                    snapshot_failure_reason: None,
                },
            )
            .await
            .unwrap();

        // Varied sizes, some spanning several `BUFFER_SIZE` reads and some
        // tiny, interleaved — a fixture where every file looked the same
        // couldn't distinguish correct framing from an entry mix-up.
        let mut files = Vec::new();
        for i in 0..24 {
            let size = match i % 3 {
                0 => 37,                    // tiny
                1 => BUFFER_SIZE + 4_096,   // just over one buffer
                _ => BUFFER_SIZE * 2 + 777, // spans several buffers
            };
            let bytes: Vec<u8> = (0..size).map(|b| (b ^ i) as u8).collect();
            files.push((format!("file-{i:02}.bin"), bytes));
        }

        let mut pool_entries = Vec::new();
        for (name, bytes) in &files {
            let mut chunk = PoolChunkWrapper::new(&config.path.pool_path, None);
            let info = chunk
                .write(
                    stream::once({
                        let bytes = bytes.clone();
                        async move { Ok(bytes) }
                    }),
                    name.as_bytes(),
                    ChunkAlgorithm::Blake3,
                    CompressionFormat::Zstd,
                )
                .await
                .unwrap();
            pool_entries.push((name.clone(), bytes.clone(), info));
        }

        let manifest = backups.get_manifest(hostname, backup_id, "/share");
        let file_entries: Vec<FileManifest> = pool_entries
            .iter()
            .map(|(name, bytes, info)| FileManifest {
                path: name.as_bytes().to_vec(),
                stats: Some(FileManifestStat {
                    file_type: FileManifestType::RegularFile as i32,
                    mode: 0o644,
                    size: bytes.len() as u64,
                    ..Default::default()
                }),
                chunks: vec![info.sha256.clone()],
                ..Default::default()
            })
            .collect();
        save_file(
            &manifest.manifest_path,
            stream::iter(file_entries),
            false,
            CompressionFormat::Zstd,
        )
        .await
        .unwrap();

        let out_dir = tmp.path().join("out");
        let output = write_host_tar_archive(
            &backups,
            &config.path.pool_path,
            hostname,
            &backup,
            &out_dir,
            ArchiveFormat::Tar(TarOptions::default()),
            None,
            CancellationToken::new(),
        )
        .await
        .unwrap();

        let file = std::fs::File::open(&output.archive_path).unwrap();
        let mut archive = tar::Archive::new(file);
        let mut seen = std::collections::HashMap::new();
        for entry in archive.entries().unwrap() {
            let mut entry = entry.unwrap();
            let declared_size = entry.header().size().unwrap();
            let path = entry.path().unwrap().to_path_buf();
            let mut body = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut body).unwrap();
            assert_eq!(
                declared_size,
                body.len() as u64,
                "header size vs. actual body length mismatch for {path:?}"
            );
            seen.insert(path, body);
        }

        assert_eq!(
            seen.len(),
            files.len(),
            "every file must appear exactly once"
        );
        for (name, bytes) in &files {
            let path = std::path::Path::new("share").join(name);
            assert_eq!(
                seen.get(&path)
                    .unwrap_or_else(|| panic!("missing entry for {path:?}")),
                bytes,
                "content mismatch for {path:?}"
            );
        }
    }

    /// Reproduces how `server-rs/src/jobs/workers.rs`'s `handle_archive_run`
    /// shares one `ArchiveProgressCounters` across a sequential run of
    /// several hosts, snapshotting it before/after each host to derive that
    /// host's own delta. Proves the counters compose correctly across calls:
    /// each host's `(bytes, files)` delta only reflects what that host
    /// itself wrote, not a running total leaking from the previous host, and
    /// each host's `archive_size` (from its own `TarArchiveOutput`) is
    /// independent of the other host's archive size.
    #[tokio::test]
    async fn shared_counters_report_independent_deltas_across_sequential_hosts() {
        let tmp = tempfile::tempdir().unwrap();
        let config = Arc::new(Configuration::from_backup_path(tmp.path().to_path_buf()));
        let backups = Backups::new(config.clone());
        let backup_id = Uuid::now_v7();
        let backup = fake_backup(backup_id);
        let out_dir = tmp.path().join("out");

        async fn write_share(
            backups: &Backups,
            hostname: &str,
            backup_id: Uuid,
            content: &[u8],
            config: &Configuration,
        ) {
            let mut chunk = PoolChunkWrapper::new(&config.path.pool_path, None);
            let info = chunk
                .write(
                    stream::once({
                        let bytes = content.to_vec();
                        async move { Ok(bytes) }
                    }),
                    b"file.txt",
                    ChunkAlgorithm::Blake3,
                    CompressionFormat::Zstd,
                )
                .await
                .unwrap();
            let dest_dir = backups.get_backup_destination_directory(hostname, backup_id);
            tokio::fs::create_dir_all(&dest_dir).await.unwrap();
            backups
                .add_backup_share_record(
                    hostname,
                    backup_id,
                    ShareRecord {
                        path: "/share".into(),
                        snapshot_method: ShareSnapshotMethod::None,
                        snapshot_failure_reason: None,
                    },
                )
                .await
                .unwrap();
            let manifest = backups.get_manifest(hostname, backup_id, "/share");
            let file_entry = FileManifest {
                path: b"file.txt".to_vec(),
                stats: Some(FileManifestStat {
                    file_type: FileManifestType::RegularFile as i32,
                    mode: 0o644,
                    size: content.len() as u64,
                    ..Default::default()
                }),
                chunks: vec![info.sha256.clone()],
                ..Default::default()
            };
            save_file(
                &manifest.manifest_path,
                stream::iter(vec![file_entry]),
                false,
                CompressionFormat::Zstd,
            )
            .await
            .unwrap();
        }

        // Different-sized content per host so a delta mix-up would produce a
        // visibly wrong (not just coincidentally equal) count/size. Chosen
        // to land in different 512-byte tar blocks (5 bytes vs. 600 bytes),
        // not just a different byte count — plain `Tar` pads each entry's
        // body up to the next 512-byte boundary, so two sizes within the
        // same block would round to an identical archive_size and defeat
        // this test's purpose.
        let host_a_content = b"short".to_vec();
        let host_b_content = vec![0x42u8; 600];
        write_share(&backups, "host-a", backup_id, &host_a_content, &config).await;
        write_share(&backups, "host-b", backup_id, &host_b_content, &config).await;

        let counters = Arc::new(ArchiveProgressCounters::default());
        let format = ArchiveFormat::Tar(TarOptions::default());

        let (host_a_start_bytes, host_a_start_files) = counters.snapshot();
        let output_a = write_host_tar_archive(
            &backups,
            &config.path.pool_path,
            "host-a",
            &backup,
            &out_dir,
            format,
            Some(counters.clone()),
            CancellationToken::new(),
        )
        .await
        .unwrap();
        let (after_a_bytes, after_a_files) = counters.snapshot();
        let host_a_bytes_delta = after_a_bytes - host_a_start_bytes;
        let host_a_files_delta = after_a_files - host_a_start_files;

        let (host_b_start_bytes, host_b_start_files) = counters.snapshot();
        let output_b = write_host_tar_archive(
            &backups,
            &config.path.pool_path,
            "host-b",
            &backup,
            &out_dir,
            format,
            Some(counters.clone()),
            CancellationToken::new(),
        )
        .await
        .unwrap();
        let (after_b_bytes, after_b_files) = counters.snapshot();
        let host_b_bytes_delta = after_b_bytes - host_b_start_bytes;
        let host_b_files_delta = after_b_files - host_b_start_files;

        assert_eq!(host_a_bytes_delta, host_a_content.len() as u64);
        assert_eq!(host_b_bytes_delta, host_b_content.len() as u64);
        assert_eq!(host_a_files_delta, 1);
        assert_eq!(host_b_files_delta, 1);

        // Each host's archive size is read back from its own archive file —
        // must differ (host B's content is longer) and neither is zero.
        assert!(output_a.archive_size > 0);
        assert!(output_b.archive_size > 0);
        assert_ne!(output_a.archive_size, output_b.archive_size);

        // The shared counters' running total is the sum of both hosts, not
        // just the last one.
        let (total_bytes, total_files) = counters.snapshot();
        assert_eq!(
            total_bytes,
            host_a_content.len() as u64 + host_b_content.len() as u64
        );
        assert_eq!(total_files, 2);
    }
}

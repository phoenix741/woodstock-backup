# Periodic Archiving

## Overview

Archiving exports the **latest `Completed` backup** of a set of hosts out of the pool onto external/removable media (a USB disk, a NAS mount, …), on a per-profile schedule. It is complementary to, and independent from, the existing extraction paths:

* `ws_restore` / the interactive restore flow — pushes a backup back onto a **live client agent**, overwriting files in place.
* `ws_console mount` (FUSE) — a read-only, **virtual** view of the pool, nothing is materialized on disk.
* The web UI's per-file ZIP download — one-off, browser-triggered, no scheduling.

Archiving instead **materializes real files** on a destination path on the server (or wherever the destination is mounted), on its own schedule, independent of a live client connection.

## Configuration (`archiving.yml`)

Profiles are configured in YAML only — there is no UI to create or edit them, by design (see below for what the UI *does* expose). The file lives next to `hosts.yml`/`scheduler.yml`, at `<config_path>/archiving.yml`, and is re-read fresh on every access (same hot-reload-by-re-read pattern as `scheduler.yml` — no restart needed to pick up changes).

```yaml
- name: usb-weekly
  hostSelection: { mode: all }
  scheduleCron: "0 0 3 * * SAT *"
  destination: /media/disqueUsb1
  format: { type: tar_gz, checksum: true, compressionLevel: 9 } # compressionLevel overrides gzip's own default (6); clamped to gzip's 0-9 range
  enabled: true

- name: nas-monthly
  hostSelection: { mode: glob, pattern: "srv-*" }
  scheduleCron: "0 0 4 1 * * *"
  destination: /mnt/nas/woodstock-archive
  format: { type: dir }
  enabled: true
```

Fields (`woodstock-rs/src/config/archiving.rs`):

| Field | Meaning |
|---|---|
| `name` | Unique profile id, also used as the on-disk/job identifier. |
| `hostSelection` | `{mode: all}`, `{mode: glob, pattern: "..."}`, `{mode: include, hosts: [...]}`, or `{mode: exclude, hosts: [...]}`. |
| `scheduleCron` | 7-field cron expression (sec min hour day month weekday year), same syntax as `nightly_schedule`. |
| `destination` | Root path the archive is written under. |
| `format` | An internally-tagged object: `{ type: dir }`, or `{ type: tar \| tar_gz \| tar_xz \| tar_zstd, checksum: bool, compressionLevel: int }` for the tar family (both `checksum` and `compressionLevel` are optional and default to `false`/omitted — they only exist on tar-family formats, `dir` cannot carry them at all). `tar_bz2` is not supported — `bzip2` isn't an enabled `async-compression` feature; add it if ever needed. |
| `enabled` | Whether the profile fires on its schedule. A disabled profile can still be triggered manually (see below). |
| `format.checksum` | Tar-family only: also write a `sha256sum`-compatible `<archive>.sha256` file. |
| `format.compressionLevel` | Tar-family only, ignored for `tar` (uncompressed). Codec-specific compression level; omit to use each codec's own recommended default (gzip/xz: `6`, zstd: `3`). Out-of-range values are clamped to the codec's valid range rather than rejected. |

## Formats

### `tar` / `tar_gz` / `tar_xz` / `tar_zstd` — `woodstock-rs/src/archiving/tar_writer.rs`

One archive file **per host**, at `<destination>/<hostname>.tar[.gz|.xz|.zst]`, streamed directly from the pool via `FileManifest::open_from_pool` — content never touches the Woodstock-proprietary compression envelope (`WoodstockCompressionWriter`'s `WSCxx` header). Output is standard, portable tar/gzip/xz/zstd, openable with ordinary `tar`/`gzip`/`xz`/`zstd` tools (`tar --zstd -xf ...` or plain `tar xf ...` on a `tar` build with zstd support). With `checksum: true`, a `sha256sum`-compatible `.sha256` file is written alongside.

Compression uses `async-compression`'s native async encoders (`GzipEncoder`/`XzEncoder`/`ZstdEncoder`), not a separate sync-encoder path — each codec's own `Level::Precise` clamping keeps a bad `compressionLevel` from failing a scheduled run. Pool-chunk reading and tar/compression writing run on **separate `spawn_blocking` threads** — by default 4 parallel reader workers plus one writer, connected by bounded `tokio::sync::mpsc` channels: a lightweight dispatcher round-robins manifest entries across the reader workers, each reader decompresses its own entries independently and hands completed ones to the single writer over a shared header channel (plus one body channel per regular file, closed by its reader at the real end of that file's content, in whatever order the readers finish — `tar::Builder` has no entry-ordering requirement, so entries land in the archive in completion order, not manifest order). This lets several files' worth of pool decompression run on different CPU cores at once, and overlaps reading with compressing/writing the current entry, so throughput can approach `min(read speed, write speed)` rather than their sum, especially when the pool and the destination are on different disks. The number of reader workers is configurable via the `WOODSTOCK_ARCHIVE_READER_WORKERS` environment variable (`woodstock-rs/src/archiving/mod.rs`'s `archive_reader_worker_count`, shared with `dir` mode below rather than duplicated per format) — a conservative static default rather than a dynamic core-count, since `job_worker` may run several archive/backup/maintenance jobs concurrently on the same machine (`ARCHIVE_CONCURRENCY` et al., `server-rs/src/jobs/config.rs`). The tar/compressor write side still batches through a 256KB `std::io::BufWriter` around its `SyncIoBridge`, for the same reason as before — batching what would otherwise be one `block_on` round-trip per ~8KB `tar::Builder` I/O call.

**Performance**: pool-chunk decompression (zstd) is CPU-bound and, per file, single-threaded — with a single reader this pegs one core while the rest sit idle, capping `tar`/`dir` throughput well below what the underlying disks can sustain even though neither read nor write is actually disk-bound (confirmed via `iostat`/`mpstat` on production hardware: max ~46% disk `%util`, one core pinned near 100% `%usr`). Parallel reader workers (above) exist to use the otherwise-idle cores for this. **This only helps formats whose writer isn't itself the bottleneck**: for `tar` (uncompressed), the writer just copies bytes and frames tar headers, so throughput scales close to linearly with the number of reader workers, up to disk bandwidth or available cores. For `tar_gz`/`tar_xz`/`tar_zstd`, the **writer** thread also runs a single-threaded compressor (see above) — that stays the bottleneck regardless of how many reader workers are added, so parallelizing the reader buys **no** throughput improvement there. `tar_zstd` at its default level is typically 5-10x faster than `tar_gz` for comparable output size, so it's the first thing to try before reaching for multithreaded compression. True multithreaded compression (zstd's `zstdmt` cargo feature, xz's `xz-parallel`) is deliberately **not** wired up yet — both require enabling a cargo feature that links pthread-dependent native code, which needs its own evaluation against this project's static/cross-compiled builds (Windows/FreeBSD, see `docs/developer_guide/README.md`) before being turned on; that's the next thing to pick up if a single `tar_gz`/`tar_xz`/`tar_zstd` profile still needs to go faster than one core allows. `dir` mode shares the same pool-decompression bottleneck and is parallelized the same way, but not with the identical dispatch: `Add`/`Modify` entries (the ones that actually read/decompress pool chunks, via `fs_materialize::materialize_entry`) fan out round-robin across `archive_reader_worker_count()` lanes on their own `spawn_blocking` threads (`dir_sync::materialize_lane`), same rationale as `tar_writer`'s reader workers. `Remove` entries are applied inline, sequentially, by the dispatcher itself instead of being fanned out too — they're cheap (no pool reads), and keeping them on a single task is what avoids needing any path-based ordering between lanes at all: a path the diff marks `Remove` can never also be the target of an `Add`/`Modify` in that same run (if anything were still live under it in the target manifest, it wouldn't be "removed"), so a directory removal's `remove_dir_all` never races a lane materializing something the diff still considers present. `dir` mode has no single-writer bottleneck analogous to `tar::Builder` — each lane writes its own independent destination file — so unlike plain `tar`, there's no reason to expect its scaling to flatten out at a fixed worker count the way the writer thread capped `tar`'s gains during testing (see above); it should keep benefiting from more lanes for longer, up to disk/CPU limits. With `checksum: true`, note that the archive is also read back in full afterwards to compute its SHA-256 — a second full pass over the output that adds to the observed run time.

A share identifier is normalized before being used as an archive path component (`woodstock::utils::path::safe_share_prefix`) — Windows shares are stored verbatim (e.g. `C:\`, see `hosts/<name>.yml`), and joining that raw string onto another path is unsafe on the Linux server: `\` isn't a path separator there, so the whole share used to collapse into one opaque component containing a literal backslash and colon, which `tar`'s stricter path validation rejects outright. It now normalizes to `C/...`. Symlink entries with an empty target (a known Windows-scanner artifact — unresolved reparse points such as WindowsApps "app execution alias" stubs, e.g. `python3.exe`, are reported as symlinks with no target) are skipped quietly rather than warned about, since there is nothing meaningful to link to.

**A regular file whose pool content can't be read at all** (a missing/unreadable chunk — this does happen: a manifest can record a hash/size for content that was never actually written to the pool during the original backup) **is skipped, logged at `error!`, and the run continues** — the archive still ends up missing that one file, but every other entry is still produced. This is safe specifically because `append_entry` peeks the body's first read (`std::io::BufRead::fill_buf`) *before* writing anything to the tar stream: if that first read fails, nothing has been committed yet, so skipping is clean. A failure that instead happens *after* the header is already flushed — mid-copy corruption, or a body that hits EOF early and copies successfully but short of the header's declared size (`tar::Builder`'s `io::copy` stops at EOF, not at the declared size) — still aborts the whole run: at that point the header already lies about this entry, and continuing would misalign every entry written after it for anything trusting the declared size. Both cases are logged distinctly (a benign path rejection or missing stats is a quiet `warn!`; unreadable content is a loud `error!`; stream corruption aborts the run and `write_host_tar_archive` deletes the partial archive).

### `dir` — `woodstock-rs/src/archiving/dir_sync.rs`

An **incremental** plain-directory mirror at `<destination>/<hostname>/<share>/...`. Each share keeps a snapshot manifest at `<destination>/<hostname>/<mangled-share>.manifest` recording the state last successfully synced there. Every run diffs that snapshot against the backup's current share manifest (the same engine as `ws_console compare`, `woodstock-rs/src/manifest/diff.rs`) and applies only the resulting `Add`/`Modify`/`Remove` — no full re-copy. The snapshot is atomically replaced only after a share's diff has been fully applied, so a failed run re-diffs from the last known-good state next time rather than from a half-applied one. Same share-prefix normalization and empty-target-symlink handling as tar-family, above.

The `dir` format restores file type + permission bits only (`woodstock-rs/src/archiving/fs_materialize.rs`) — no xattr/ACL/device-node/ownership fidelity, unlike the full client-agent restore path. It targets disaster-recovery-by-hand off a USB/NAS mount, not a bit-for-bit production restore.

### Chunk lifetime

Both formats copy bytes **out of** the pool onto external media; they never leave chunks referenced only inside the pool. No new REFCNT/garbage-collection interaction is needed — an archived host's chunks are read via the normal `open_from_pool` path before the source backup might later be pruned.

## Execution pipeline

1. **`scheduler` binary** — checked on every iteration of the unified dynamic-wakeup scanner loop (`check_and_enqueue_due_archives`, `server-rs/bin/scheduler.rs`), not on a fixed tick: it loads `archiving.yml`, and for each `enabled` profile checks due-ness (`woodstock::utils::cron_due::next_due_at`, comparing the profile's cron schedule against `<jobs_path>/archiving/<profile>.yml`'s recorded `last_run`) — the same function also feeds the scanner's sleep calculation, so it wakes up exactly when a profile becomes due rather than polling. A due profile's host selection is resolved once and fanned into a **single** `ArchiveJobData::Run{profile_name, hostnames}` job covering every selected host (`Producers::enqueue_archive_profile`) — deliberately not one job per host, so a profile drives its destination disk (often removable media) with one write stream at a time instead of seeking across it from N concurrent jobs.
2. **Queue** — a dedicated Apalis/Redis queue (`QueueName::Archive`), separate from backup/restore/maintenance.
3. **`job_worker` binary** — the `archive-worker` consumes the queue and, inside `handle_archive_run`, archives `hostnames` **sequentially**: for each host it resolves the latest `Completed` backup (explicitly filtering on `BackupStatus::Completed`, not `get_last_backup`/`is_finished()` — those don't exclude `Aborted`/`Failed`), takes a **shared** host lock (`HostLockOperation::Archive` — blocks on a concurrent Backup/Restore/Remove, but multiple archive reads of the same host may run concurrently) for just that host, and dispatches to `tar_writer` or `dir_sync`. A single host failing (no completed backup, lock timeout, write error) is recorded and logged but does not fail the job — the queue's `RetryPolicy` retries a failed job in full, which would re-archive every host that already succeeded, so failures are surfaced through progress (`failedHosts`, below) instead of an `Err` that would trigger a redundant, disk-thrashing retry.

### Progress

Progress is real and byte-based, not the coarse per-job-status guess used before: `woodstock::archiving::ArchiveState` tracks `currentHost`, `hostsDone`/`hostsTotal`, `progressCurrent`/`progressMax` (bytes), and `failedHosts`, published through the same Redis pub/sub `ProgressPublisher` pipeline as backup/restore jobs (`ProgressUpdate::Archive`).

`progressMax` is derived once, before the first host starts: `Backup::file_size` (already-recorded metadata, no extra pass) for tar-family, or the sum of `Add`/`Modify` diff-journal entry sizes (`dir_sync::dir_diff_total_size`) for `dir` mode — `file_size` there would count the *whole* backup rather than the delta a `dir` run actually writes, which would leave a near-no-op sync sitting at 0% before jumping to 100%. `progressCurrent` is a plain `Arc<AtomicU64>` incremented per-file from inside the writer (which runs on the blocking pool for tar-family, so a channel send would need `blocking_send` for no benefit at file-level granularity); a ~1s ticker on the async side reads it and publishes a snapshot while each host's write is in flight, and the worker publishes an exact snapshot itself at every host boundary.

## Manual trigger (CLI, UI, USB hotplug)

`ws_console archive run <profile> [--host <h>]` runs a profile immediately, regardless of whether it's `enabled` or due. It is the single code path used by:

* an administrator at the CLI,
* the "Run now" button on the dedicated Archive page (GraphQL `runArchive(profile, host)` mutation — enabled state is intentionally ignored here too, same as the existing "launch backup" button ignores a disabled schedule),
* a udev rule triggered by connecting a USB disk:

```
# /etc/udev/rules.d/99-woodstock-archive.rules
ACTION=="add", SUBSYSTEM=="block", ENV{ID_FS_LABEL}=="WOODSTOCK_USB1", \
  RUN+="/usr/bin/systemd-run --no-block /usr/local/bin/ws_console archive run usb-weekly"
```

There is no device-detection code in Woodstock itself — hotplug and scheduled runs share the exact same command and code path, so there is nothing hotplug-specific to test beyond the udev rule.

## `ws_console archive` subcommands

* `list` — configured profiles plus last-run timestamp (from `<jobs_path>/archiving/<profile>.yml`).
* `run <profile> [--host <h>]` — manual trigger (see above).
* `diff <profile> <hostname>` — dry-runs a `dir`-mode profile's next sync: prints Add/Modify/Remove without touching the destination.
* `verify <profile> <hostname>` — recomputes a tar-family archive's SHA-256 and compares it against its `.sha256` file.

## UI

There are three places archiving shows up in the frontend, mirroring the read-only-config decision above:

* **Archive list** (`front/src/views/ArchiveView.vue`, nav bar between Devices and Tasks) — a data table of profiles (name, format, destination, host-selection summary, schedule, enabled state), styled like the Devices list. Clicking a row navigates to that profile's detail page.
* **Archive detail** (`front/src/views/ArchiveProfileView.vue` + `front/src/components/archiving/ArchiveProfileCard.vue`, route `/archive/:profileName`) — a read-only header card (styled like a host's detail header, plain text/chips, not form inputs) showing the full profile configuration, with `hostSelection` broken out into separate fields per mode (`All hosts` / a `Filter (glob pattern)` field / an `Include hosts` chip list / an `Exclude hosts` chip list) rather than one opaque blob. A single "Run now" button opens a dialog asking for an optional host-override *at the point of triggering*, following the same Waiting/InProgress/Success/Error dialog pattern as "Launch backup" (`runArchive` mutation) — enabled state is ignored, so a disabled profile's button still works.
* **Tasks page** — Archive jobs appear like any other job kind (`JobKind::Archive`), including live status via the same `jobUpdated` GraphQL subscription used by backup/restore/fsck/cleanup. Since a run is a single job spanning every selected host (see Execution pipeline, above), the task card shows real byte-based progress plus which host is currently being archived, an N/M hosts-done count, and a warning chip listing any hosts that failed.

Profile configuration itself is not editable from the UI in any of these — creating/editing profiles is YAML-only (see above).

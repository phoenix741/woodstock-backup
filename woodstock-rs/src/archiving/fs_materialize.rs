//! Lightweight, archive-oriented filesystem materializer for the incremental
//! `dir` archiving format.
//!
//! Special nodes, symlinks, and permission bits are restored via
//! [`crate::utils::restore_metadata`] — the same logic `client-rs`'s
//! `create_file_from_manifest` uses for a real restore, so the two can never
//! diverge on e.g. which `stats` field a device node's major/minor comes
//! from. For regular files, xattrs/ACLs are also attempted best-effort
//! (warned, not fatal, on failure) via the same shared module — not for
//! directories (`restore_xattr` opens with `O_WRONLY`, which fails with
//! `EISDIR`), symlinks (chmod/xattr would follow the link), or special
//! nodes (opening a real FIFO for write-only blocks until a reader
//! connects, with none here to provide one). Ownership (uid/gid) and
//! timestamps are restored by neither side and remain out of scope.
//! `dir`-mode archives are for disaster-recovery-by-hand off a USB/NAS
//! mount, not necessarily a bit-for-bit production restore, so a
//! metadata-restore failure here must never abort materializing the file's
//! actual content. Unix sockets are skipped entirely (`mknod` doesn't
//! support `S_IFSOCK` on Linux, and a socket has no meaningful on-disk
//! content to recreate anyway).

use std::path::Path;

use eyre::Result;
use tracing::{debug, warn};

use crate::utils::path::vec_to_path;
use crate::utils::restore_metadata;
use crate::{FileManifest, FileManifestType, SourceOs};

/// Materializes one manifest entry at `path` (already rewritten to point
/// inside the destination tree — see `dir_sync`), creating parent
/// directories as needed and streaming regular-file content from the pool.
///
/// # Errors
/// Returns an error if the filesystem entry cannot be created or written.
pub async fn materialize_entry(entry: &FileManifest, path: &Path, pool_path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    match entry.file_mode() {
        FileManifestType::Directory => {
            tokio::fs::create_dir_all(path).await?;
            // Permissions are deliberately NOT applied here: some captured
            // source directories carry owner-only modes with no execute bit
            // (e.g. `iscsi` node directories at 0o600), and chmod'ing a
            // directory that restrictively before its children are written
            // would lock ourselves out of it. The caller (`dir_sync`) defers
            // this to a bottom-up pass after the whole share is materialized
            // — see `set_directory_permissions`.
            //
            // Not attempting xattr/ACL restoration here either:
            // `restore_xattr` opens the target with `OpenOptions::write(true)`
            // (needed for FreeBSD's `extattr_set_fd`), which fails with
            // `EISDIR` for a directory on Unix — every attempt would just
            // warn, never succeed.
        }
        FileManifestType::Symlink => {
            if entry.symlink.is_empty() {
                // Known Windows scanner artifact: unresolved reparse points
                // (e.g. WindowsApps "app execution alias" stubs) are reported
                // as symlinks with no target. There is nothing meaningful to
                // link to, so skip quietly rather than fail the whole sync.
                debug!("Skipping symlink with empty target: {path:?}");
                return Ok(());
            }
            let target = vec_to_path(&entry.symlink);
            // Remove a possibly-stale existing entry (file/symlink) before recreating.
            let _ = tokio::fs::remove_file(path).await;
            #[cfg(unix)]
            tokio::fs::symlink(&target, path).await?;
            // Matches `client-rs`'s `metadata::windows::create_symlink`:
            // Windows distinguishes file vs. directory symlinks and
            // `std::os::windows::fs` has no manifest-driven way to know
            // which the target is, so this only creates file symlinks.
            #[cfg(windows)]
            std::os::windows::fs::symlink_file(&target, path)?;
            // Not calling set_permissions here: chmod follows symlinks, so it
            // would silently affect the link's target (or fail on a
            // dangling one) rather than the link itself.
        }
        FileManifestType::RegularFile | FileManifestType::Unknown => {
            let mut file = open_for_write_retrying_on_eacces(path).await?;

            let reader = entry.open_from_pool(pool_path);
            tokio::pin!(reader);
            tokio::io::copy(&mut reader, &mut file).await?;

            set_permissions(path, entry).await;
            restore_xattr_and_acl(path, entry).await;
        }
        FileManifestType::Fifo
        | FileManifestType::BlockDevice
        | FileManifestType::CharacterDevice => {
            // Remove a possibly-stale existing entry before recreating, same
            // as the `Symlink` arm above — `mknod` fails with `EEXIST`
            // otherwise on a re-sync (including when a directory sits at
            // `path`, which `remove_file` above can't clear).
            let _ = tokio::fs::remove_file(path).await;
            // Unlike every other arm, a failure here is not propagated with
            // `?`: `mknod` for a block/char device needs `CAP_MKNOD`, which
            // an unprivileged worker won't have, and per `dir_sync.rs`'s own
            // doc comment on this call site, a propagated error here aborts
            // the *entire* share sync. A missing device node must degrade
            // the same way `tar_writer`'s equivalent case does — skip this
            // one entry, keep going — not take down an otherwise-successful
            // disaster-recovery copy over one `/dev` entry.
            if let Err(err) = create_special_node(path, entry).await {
                warn!("Skipping special node {path:?}: {err}");
                return Ok(());
            }
            set_permissions(path, entry).await;
            // Not restoring xattr/ACL here: `restore_xattr` opens the node
            // for writing, and opening a real FIFO for write-only blocks
            // until a reader connects — there is none here, so this would
            // hang the whole sync rather than just skip one entry's
            // metadata.
        }
        FileManifestType::Socket => {
            // `mknod(2)` doesn't support `S_IFSOCK` on Linux (returns
            // `EINVAL`), and a socket has no meaningful on-disk content to
            // recreate here anyway — skipped, same choice as `tar_writer`'s
            // archive path.
            debug!("Skipping socket (cannot be recreated via mknod): {path:?}");
        }
    }

    Ok(())
}

/// Recreates a block/character device, FIFO, or socket node at `path` via
/// [`restore_metadata::mknode`] — the same function `client-rs`'s real
/// restore path uses, so both agree on using `entry.stats.rdev` (the node's
/// own major/minor) rather than `stats.dev` (the containing filesystem's
/// device, a different field an earlier duplicate of this logic in
/// `client-rs` used by mistake).
///
/// # Errors
/// Returns an error if `mknod` fails (permissions, an unsupported entry
/// type, `CAP_MKNOD` missing for a block/char device, etc), or if this
/// platform has no `mknod` at all (see below).
async fn create_special_node(path: &Path, entry: &FileManifest) -> Result<()> {
    // `restore_metadata::mknode` no-ops (`Ok(())`) on non-Unix, matching
    // `client-rs`'s pre-existing restore behavior there. That's fine for
    // client-rs, which never restores device/FIFO entries onto a Windows
    // target. Here the caller (this function's `Err` arm above) treats a
    // failure as "skip this one entry" but a *success* as "node applied" —
    // feeding the no-op stub's `Ok(())` through unchanged would make the
    // caller believe a node was created when nothing happened on disk, the
    // exact silent-loss shape this refactor was meant to close elsewhere.
    #[cfg(not(unix))]
    {
        let _ = (path, entry);
        return Err(eyre::eyre!(
            "special device nodes are not supported on this platform"
        ));
    }

    #[cfg(unix)]
    {
        let path = path.to_path_buf();
        let entry = entry.clone();

        tokio::task::spawn_blocking(move || restore_metadata::mknode(&path, &entry)).await?
    }
}

/// Removes the file/directory at `path` (recursively for directories).
/// A missing path is not an error — the entry may already have been removed
/// by a previous, partially-applied sync.
///
/// # Errors
/// Returns an error if the entry exists but cannot be removed.
pub async fn remove_entry(path: &Path) -> Result<()> {
    let metadata = match tokio::fs::symlink_metadata(path).await {
        Ok(metadata) => metadata,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
    };

    if metadata.is_dir() {
        tokio::fs::remove_dir_all(path).await?;
    } else {
        tokio::fs::remove_file(path).await?;
    }

    Ok(())
}

/// Async wrapper around [`crate::utils::files::open_for_write_retrying_on_eacces`]
/// (shared with `client-rs`'s synchronous real-restore path) — run via
/// `spawn_blocking` since the retry does a blocking `chmod` on the failure
/// path.
async fn open_for_write_retrying_on_eacces(path: &Path) -> Result<tokio::fs::File> {
    let owned_path = path.to_path_buf();
    let std_file =
        tokio::task::spawn_blocking(move || crate::utils::files::open_for_write_retrying_on_eacces(&owned_path))
            .await??;
    Ok(tokio::fs::File::from_std(std_file))
}

async fn set_permissions(path: &Path, entry: &FileManifest) {
    set_mode(path, entry.mode(), entry.source_os()).await;
}

/// Attempts to restore `entry`'s extended attributes and ACLs onto `path`
/// via [`restore_metadata::xattr`]/[`restore_metadata::acl`] — the same
/// functions `client-rs`'s real restore path uses. Unlike a missing pool
/// chunk or a failed `mknod`, a failure here only ever warns: per the PR
/// #107 review, it's fine for `dir`-mode archiving to end up with degraded
/// xattr/ACL fidelity, but it must still *try*, and it must never abort
/// materializing the entry's actual content/type over metadata that
/// couldn't be applied (e.g. the destination filesystem not supporting
/// xattrs at all).
async fn restore_xattr_and_acl(path: &Path, entry: &FileManifest) {
    let owned_path = path.to_path_buf();
    let xattr = entry.xattr.clone();
    let acl = entry.acl.clone();

    let result = tokio::task::spawn_blocking(move || {
        if let Err(err) = restore_metadata::xattr::restore_xattr(&owned_path, &xattr) {
            warn!("Failed to restore xattr on {owned_path:?}: {err}");
        }
        if let Err(err) = restore_metadata::acl::restore_acl(&owned_path, &acl) {
            warn!("Failed to restore acl on {owned_path:?}: {err}");
        }
    })
    .await;

    if let Err(err) = result {
        warn!("Restoring xattr/acl on {path:?} panicked: {err}");
    }
}

/// Applies `mode`'s permission bits (masked to 0o777) to `path`, via
/// [`restore_metadata::restore_permissions`] — the same function
/// `client-rs`'s real restore path uses.
///
/// Used by [`materialize_entry`] for files/symlinks right after creation,
/// and by `dir_sync` for directories in a deferred, bottom-up pass — see
/// [`set_directory_permissions`].
async fn set_mode(path: &Path, mode: u32, source_os: SourceOs) {
    let owned_path = path.to_path_buf();
    let result = tokio::task::spawn_blocking(move || {
        restore_metadata::restore_permissions(&owned_path, mode, source_os)
    })
    .await;
    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => warn!("Failed to set permissions on {path:?}: {e}"),
        Err(e) => warn!("Setting permissions on {path:?} panicked: {e}"),
    }
}

/// Applies a directory's captured permission bits to `path`, once all of its
/// contents have already been materialized (see `dir_sync::sync_host_dir_archive`,
/// which calls this bottom-up — deepest directories first — after a share's
/// diff has been fully applied).
///
/// Unlike [`set_permissions`], this always keeps the owner's read/write/execute
/// bits regardless of the captured mode: `dir`-mode archives are a
/// disaster-recovery-by-hand copy (see module docs), and a captured mode that
/// happens to omit the owner's execute bit (some real-world directories do,
/// e.g. iscsi node configs) must never lock the archive's own owner out of
/// their own copy. Still subject to the same source/target OS gating as
/// [`set_permissions`] — see [`restore_metadata::restore_permissions`].
pub async fn set_directory_permissions(path: &Path, mode: u32, source_os: SourceOs) {
    set_mode(path, mode | 0o700, source_os).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FileManifestStat;

    fn fifo_entry() -> FileManifest {
        FileManifest {
            path: b"fifo".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::Fifo as i32,
                mode: 0o644,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn socket_entry() -> FileManifest {
        FileManifest {
            path: b"socket".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::Socket as i32,
                mode: 0o755,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    /// Proves `materialize_entry` now recreates a real FIFO node via `mknod`
    /// (instead of silently dropping it through the old catch-all).
    #[cfg(unix)]
    #[tokio::test]
    async fn materializes_fifo_as_real_fifo_node() {
        use std::os::unix::fs::FileTypeExt;

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("fifo");
        let pool_path = tmp.path().join("pool");

        materialize_entry(&fifo_entry(), &path, &pool_path)
            .await
            .unwrap();

        let metadata = tokio::fs::symlink_metadata(&path).await.unwrap();
        assert!(metadata.file_type().is_fifo());
    }

    /// `mknod` for a block/char device needs `CAP_MKNOD`, which the test
    /// runner (like a real unprivileged worker) may not have — proves that
    /// case degrades to a skipped entry rather than aborting the whole sync
    /// with `?`, regardless of whether it happens to succeed here as root.
    #[cfg(unix)]
    #[tokio::test]
    async fn materialize_never_fails_for_a_block_device_regardless_of_privilege() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("block");
        let pool_path = tmp.path().join("pool");

        let entry = FileManifest {
            path: b"block".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::BlockDevice as i32,
                mode: 0o660,
                rdev: (8u64 << 8) | 1u64,
                ..Default::default()
            }),
            ..Default::default()
        };

        let result = materialize_entry(&entry, &path, &pool_path).await;
        assert!(
            result.is_ok(),
            "a permission-denied mknod must be skipped, not abort the sync: {result:?}"
        );
    }

    /// A socket has no on-disk representation `mknod` can recreate — proves
    /// it's skipped cleanly (no error, no file left behind) rather than
    /// attempted and failing.
    #[tokio::test]
    async fn materialize_skips_socket_without_creating_anything() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("socket");
        let pool_path = tmp.path().join("pool");

        materialize_entry(&socket_entry(), &path, &pool_path)
            .await
            .unwrap();

        assert!(
            tokio::fs::symlink_metadata(&path).await.is_err(),
            "no file should have been created for a socket entry"
        );
    }

    fn regular_file_entry(mode: u32, source_os: SourceOs) -> FileManifest {
        FileManifest {
            path: b"regular".to_vec(),
            stats: Some(FileManifestStat {
                file_type: FileManifestType::RegularFile as i32,
                mode,
                source_os: source_os as i32,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[cfg(unix)]
    fn unix_mode_bits(path: &std::path::Path) -> u32 {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    /// Prevents the regression from ever occurring: a Windows-sourced
    /// entry's `mode` is a raw `FILE_ATTRIBUTE_*` bitmask (here
    /// `FILE_ATTRIBUTE_ARCHIVE = 32`, masking to `0o040` — no owner bits at
    /// all), not POSIX bits. Materializing it must NOT chmod the
    /// destination to that garbage value — otherwise a later re-sync's
    /// `OpenOptions::write(true).open()` on the already-existing file fails
    /// `EACCES` forever, exactly as seen in production (`+739 ~0 -19
    /// (skipped 175)`, every skip a failed re-materialization). This only
    /// covers files materialized *after* this fix ships — see
    /// [`re_materializing_an_already_corrupted_file_still_fails`] for the
    /// known limitation on files a pre-fix run already broke.
    #[cfg(unix)]
    #[tokio::test]
    async fn windows_sourced_mode_is_not_applied_on_unix_target() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("regular");
        let pool_path = tmp.path().join("pool");

        let entry = regular_file_entry(32, SourceOs::Windows);
        materialize_entry(&entry, &path, &pool_path).await.unwrap();

        let mode = unix_mode_bits(&path);
        assert_ne!(
            mode, 0o040,
            "Windows FILE_ATTRIBUTE_* bits must never be chmod'd onto a Unix target"
        );
        assert_eq!(
            mode & 0o200,
            0o200,
            "owner-write bit must survive, or a later re-sync can never re-open this file for writing"
        );

        // Re-materializing (simulating a re-sync after the source changed)
        // must still succeed, since this file was never corrupted to begin
        // with under the fixed code.
        materialize_entry(&entry, &path, &pool_path)
            .await
            .expect("re-materializing an existing file must not fail with Permission denied");
    }

    /// A destination already stuck at a restrictive mode — whether garbage
    /// left by a pre-fix run's Windows-mode chmod, or (see the sibling test
    /// below) a legitimate real Unix mode — must self-heal instead of
    /// failing `EACCES` forever: `materialize_entry` retries the open after
    /// chmod'ing the file writable once the first attempt hits
    /// `PermissionDenied`, so no manual `chmod -R u+w <dest>` remediation is
    /// needed after this fix ships (unlike before it did).
    ///
    /// Skipped as root: root bypasses the DAC check that makes opening a
    /// `0o040` file for write hit `PermissionDenied` in the first place, so
    /// the retry/unlock path this test exercises never triggers — same as
    /// every other permission-check test in this crate (see `dir_sync.rs`).
    #[cfg(unix)]
    #[tokio::test]
    async fn re_materializing_a_file_stuck_at_a_restrictive_mode_now_succeeds() {
        use std::os::unix::fs::PermissionsExt;

        // SAFETY: geteuid() takes no arguments and has no preconditions.
        if unsafe { libc::geteuid() } == 0 {
            eprintln!("skipping: root bypasses the permission check this test relies on");
            return;
        }

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("regular");
        let pool_path = tmp.path().join("pool");

        // Simulate a file left behind by a pre-fix run: content exists, but
        // the destination's mode was already corrupted to 0o040 (no owner
        // bits) by the old, unconditional chmod.
        std::fs::write(&path, b"old content").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o040)).unwrap();

        let entry = regular_file_entry(32, SourceOs::Windows);
        materialize_entry(&entry, &path, &pool_path)
            .await
            .expect("a restrictive existing mode must not block re-materializing");

        assert_eq!(
            std::fs::read(&path).unwrap(),
            b"",
            "content must actually have been overwritten, not just the open() call succeeding"
        );
        // source_os is Windows, so restore_permissions skips chmod'ing the
        // final mode — it stays at whatever the retry's temporary unlock set
        // it to, which must at least keep the file writable for the next run.
        assert_eq!(unix_mode_bits(&path) & 0o200, 0o200);
    }

    /// Same self-healing behavior, but for a legitimate real Unix mode that
    /// blocks a rewrite — not a Windows-mode bug artifact. Git leaves its
    /// loose objects and packs at `0o444` (read-only, no owner-write); a
    /// `dir`-archive destination mirroring a git bare repo hits the exact
    /// same `EACCES`-on-reopen wall as the Windows case, purely from a
    /// correct, intentional source permission (this is the `server-ovh-6`
    /// production incident: a Linux host, `source_os` already correctly
    /// `Unix`, failing on `.git/objects/**`).
    #[cfg(unix)]
    #[tokio::test]
    async fn re_materializing_a_file_at_a_legitimate_readonly_unix_mode_now_succeeds() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("regular");
        let pool_path = tmp.path().join("pool");

        std::fs::write(&path, b"old git object content").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o444)).unwrap();

        let entry = regular_file_entry(0o444, SourceOs::Unix);
        materialize_entry(&entry, &path, &pool_path)
            .await
            .expect("a legitimate real-Unix restrictive mode must not block re-materializing");

        assert_eq!(std::fs::read(&path).unwrap(), b"");
        // source_os is Unix, so restore_permissions re-applies the real
        // final mode afterward — the file must end up back at 0o444, not
        // stuck at the retry's temporary 0o600 unlock.
        assert_eq!(unix_mode_bits(&path), 0o444);
    }

    /// Non-regression: a Unix-sourced entry's captured mode is still applied
    /// as before.
    #[cfg(unix)]
    #[tokio::test]
    async fn unix_sourced_mode_is_still_applied_on_unix_target() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("regular");
        let pool_path = tmp.path().join("pool");

        let entry = regular_file_entry(0o600, SourceOs::Unix);
        materialize_entry(&entry, &path, &pool_path).await.unwrap();

        assert_eq!(unix_mode_bits(&path), 0o600);
    }

    /// A Windows-sourced directory's captured `FILE_ATTRIBUTE_*` bits must
    /// not be chmod'd onto the destination directory either.
    #[cfg(unix)]
    #[tokio::test]
    async fn windows_sourced_directory_permission_is_not_applied_on_unix_target() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("dir");
        std::fs::create_dir_all(&path).unwrap();
        let before = unix_mode_bits(&path);

        set_directory_permissions(&path, 32, SourceOs::Windows).await;

        assert_eq!(
            unix_mode_bits(&path),
            before,
            "a Windows-sourced directory mode must not be chmod'd onto a Unix target"
        );
    }
}

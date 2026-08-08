//! Lightweight, archive-oriented filesystem materializer for the incremental
//! `dir` archiving format.
//!
//! Deliberately simpler than `client-rs`'s `create_file_from_manifest`: it
//! only restores file type + permission bits (0o777, matching
//! `client-rs::scanner::metadata::unix::restore_permissions`'s existing
//! scope) — no xattr/ACL/ownership fidelity. `dir`-mode archives are for
//! disaster-recovery-by-hand off a USB/NAS mount, not a bit-for-bit
//! production restore. Device nodes/FIFOs are recreated via `libc::mknod`
//! (Unix only) — `libc` is already a `woodstock-rs` dependency, so this adds
//! no new crate to the dependency graph; Unix sockets are skipped (`mknod`
//! doesn't support `S_IFSOCK` on Linux, and a socket has no meaningful
//! on-disk content to recreate anyway).

use std::path::Path;

use eyre::{eyre, Result};
use tokio::fs::OpenOptions;
use tracing::{debug, warn};

use crate::utils::path::vec_to_path;
use crate::{FileManifest, FileManifestType};

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
            tokio::fs::symlink(&target, path).await?;
            // Not calling set_permissions here: chmod follows symlinks, so it
            // would silently affect the link's target (or fail on a
            // dangling one) rather than the link itself.
        }
        FileManifestType::RegularFile | FileManifestType::Unknown => {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)
                .await?;

            let reader = entry.open_from_pool(pool_path);
            tokio::pin!(reader);
            tokio::io::copy(&mut reader, &mut file).await?;

            set_permissions(path, entry).await;
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

/// Recreates a block/character device or FIFO node at `path` via `mknod`,
/// using `entry.stats.rdev` (the raw `dev_t` captured by the scanning
/// client) verbatim as `mknod`'s device argument — unlike
/// `client-rs::scanner::metadata::unix::mknode`, which passes `stats.dev`
/// (the filesystem's own device, not the node's) by mistake.
///
/// # Errors
/// Returns an error if `mknod` fails (permissions, an unsupported entry
/// type, or — on non-Unix targets — unconditionally, since there is no
/// equivalent there).
#[cfg(unix)]
async fn create_special_node(path: &Path, entry: &FileManifest) -> Result<()> {
    use libc::{dev_t, mknod, mode_t, S_IFBLK, S_IFCHR, S_IFIFO};
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let mode_filter = match entry.file_mode() {
        FileManifestType::BlockDevice => S_IFBLK,
        FileManifestType::CharacterDevice => S_IFCHR,
        FileManifestType::Fifo => S_IFIFO,
        other => {
            return Err(eyre!(
                "create_special_node called for unsupported file type {other:?}"
            ))
        }
    };
    let rdev = entry.stats.as_ref().map_or(0, |stats| stats.rdev);
    let mode = entry.mode() as mode_t | mode_filter;
    let path = path.to_path_buf();

    tokio::task::spawn_blocking(move || -> Result<()> {
        let c_path = CString::new(path.as_os_str().as_bytes())
            .map_err(|err| eyre!("path {path:?} contains a NUL byte: {err}"))?;
        // SAFETY: `c_path` is a valid, NUL-terminated C string kept alive for
        // the whole call; `mknod` only reads it and never retains the
        // pointer past this call.
        let result = unsafe { mknod(c_path.as_ptr(), mode, rdev as dev_t) };
        if result != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(())
    })
    .await?
}

#[cfg(not(unix))]
async fn create_special_node(path: &Path, _entry: &FileManifest) -> Result<()> {
    Err(eyre!(
        "cannot create device/FIFO node at {path:?}: unsupported on this platform"
    ))
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

async fn set_permissions(path: &Path, entry: &FileManifest) {
    set_mode(path, entry.mode()).await;
}

/// Applies `mode`'s permission bits (masked to 0o777) to `path`.
///
/// Used by [`materialize_entry`] for files/symlinks right after creation,
/// and by `dir_sync` for directories in a deferred, bottom-up pass — see
/// [`set_directory_permissions`].
async fn set_mode(path: &Path, mode: u32) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = mode & 0o777;
        if let Err(e) =
            tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).await
        {
            warn!("Failed to set permissions on {path:?}: {e}");
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (path, mode);
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
/// their own copy.
pub async fn set_directory_permissions(path: &Path, mode: u32) {
    set_mode(path, mode | 0o700).await;
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
}

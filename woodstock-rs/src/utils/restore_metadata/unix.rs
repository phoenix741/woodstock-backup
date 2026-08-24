//! Unix implementation of restore-side metadata operations: special device
//! nodes, symlinks, and permission bits.

use eyre::Result;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::{FileManifest, FileManifestType, SourceOs};

/// Creates a block/character device, FIFO, or socket node at `path` via
/// `mknod`, based on `entry`'s manifest type and permission bits, using
/// `entry.stats.rdev` — the node's own major/minor device number — as
/// `mknod`'s device argument. Not `stats.dev` (the containing filesystem's
/// own device, a different field entirely): an earlier, now-removed
/// duplicate of this function in `client-rs` used `stats.dev` by mistake,
/// which produced device nodes with the wrong major/minor on restore.
///
/// # Errors
/// Returns an error if `mknod` fails (permissions, `CAP_MKNOD` missing for a
/// block/char device, `path` contains a NUL byte, etc).
pub fn mknode(path: &Path, entry: &FileManifest) -> Result<()> {
    use libc::{dev_t, mknod, mode_t, S_IFBLK, S_IFCHR, S_IFIFO, S_IFSOCK};
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let mode_filter = match entry.file_mode() {
        FileManifestType::BlockDevice => S_IFBLK,
        FileManifestType::CharacterDevice => S_IFCHR,
        FileManifestType::Fifo => S_IFIFO,
        FileManifestType::Socket => S_IFSOCK,
        _ => 0,
    };

    let rdev = entry.stats.as_ref().map_or(0, |stats| stats.rdev);
    let mode = entry.mode() as mode_t | mode_filter;
    // `CString::new` NUL-terminates (and rejects an embedded NUL) rather than
    // handing `mknod` a bare, non-terminated byte slice pointer.
    let c_path = CString::new(path.as_os_str().as_bytes())
        .map_err(|err| eyre::eyre!("path {path:?} contains a NUL byte: {err}"))?;
    // SAFETY: `c_path` is a valid, NUL-terminated C string kept alive for the
    // whole call; `mknod` only reads it and never retains the pointer past
    // this call.
    let result = unsafe { mknod(c_path.as_ptr(), mode, rdev as dev_t) };
    if result != 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    Ok(())
}

/// Creates a symbolic link at `path` pointing to `target`.
///
/// # Errors
/// Returns an error if the symlink cannot be created.
pub fn create_symlink<P: AsRef<Path>>(path: P, target: P) -> Result<()> {
    std::os::unix::fs::symlink(target, path)?;

    Ok(())
}

/// Applies `mode`'s permission bits (masked to `0o777`, ignoring any file
/// type bits a raw `st_mode`-derived value may also carry) to `path` — but
/// only when `source_os` is [`SourceOs::Unix`] (or [`SourceOs::Unspecified`],
/// for manifests captured before this field existed — treated as "assume
/// same as target" so already-correct legacy Unix backups keep restoring as
/// before). [`SourceOs::Windows`] is skipped entirely: `mode` there is a raw
/// `FILE_ATTRIBUTE_*` bitmask, not POSIX bits, and blindly chmod'ing it
/// produces near-arbitrary permissions (e.g. `FILE_ATTRIBUTE_ARCHIVE = 32`
/// masks to `0o040`, stripping every owner bit) that can permanently lock a
/// later re-sync out of its own file with `EACCES`. The file/dir content
/// itself is still created/written by the caller regardless of this
/// function's outcome — skipping here only means the destination keeps
/// whatever default permissions its creation left it with.
///
/// Takes a plain `mode` rather than a `&FileManifest` since some callers
/// (e.g. `dir_sync`'s deferred, bottom-up directory-permission pass) only
/// have a possibly-adjusted mode value on hand at the point they call this,
/// not the original manifest entry.
///
/// Sets permissions by path (`chmod`), not by opening the file first: a
/// real FIFO/device node has no counterpart to pair with, so opening it
/// (even read-only) would block indefinitely waiting for one — `chmod`
/// needs no open file descriptor at all.
///
/// # Errors
/// Returns an error if `path` does not exist or its permissions cannot be
/// set.
pub fn restore_permissions<P: AsRef<Path>>(path: P, mode: u32, source_os: SourceOs) -> Result<()> {
    if source_os == SourceOs::Windows {
        return Ok(());
    }

    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode & 0o777))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn current_mode(path: &Path) -> u32 {
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn skips_a_windows_sourced_mode() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::set_permissions(tmp.path(), std::fs::Permissions::from_mode(0o644)).unwrap();

        // FILE_ATTRIBUTE_ARCHIVE = 32, masks to 0o040 — must never be applied.
        restore_permissions(tmp.path(), 32, SourceOs::Windows).unwrap();

        assert_eq!(current_mode(tmp.path()), 0o644, "mode must be left untouched");
    }

    #[test]
    fn applies_a_unix_sourced_mode() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::set_permissions(tmp.path(), std::fs::Permissions::from_mode(0o644)).unwrap();

        restore_permissions(tmp.path(), 0o600, SourceOs::Unix).unwrap();

        assert_eq!(current_mode(tmp.path()), 0o600);
    }

    #[test]
    fn applies_an_unspecified_source_mode_for_legacy_manifests() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::set_permissions(tmp.path(), std::fs::Permissions::from_mode(0o644)).unwrap();

        restore_permissions(tmp.path(), 0o600, SourceOs::Unspecified).unwrap();

        assert_eq!(current_mode(tmp.path()), 0o600);
    }
}

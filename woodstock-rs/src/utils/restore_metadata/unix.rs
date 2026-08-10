//! Unix implementation of restore-side metadata operations: special device
//! nodes, symlinks, and permission bits.

use eyre::Result;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::{FileManifest, FileManifestType};

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
/// type bits a raw `st_mode`-derived value may also carry) to `path`.
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
pub fn restore_permissions<P: AsRef<Path>>(path: P, mode: u32) -> Result<()> {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode & 0o777))?;

    Ok(())
}

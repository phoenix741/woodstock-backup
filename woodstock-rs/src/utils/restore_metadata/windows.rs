//! Windows stub implementation of restore-side metadata operations.
//!
//! Windows has no equivalent of `mknod`-style special nodes or POSIX
//! permission bits, so [`mknode`] and [`restore_permissions`] are no-ops —
//! only [`create_symlink`] does real work.

use eyre::Result;
use std::path::Path;

use crate::{FileManifest, SourceOs};

/// No-op on Windows: there is no equivalent of a Unix special device/FIFO
/// node to create.
///
/// # Errors
/// Never returns an error.
pub fn mknode(_path: &Path, _entry: &FileManifest) -> Result<()> {
    Ok(())
}

/// Creates a file symbolic link at `path` pointing to `target`. Windows
/// distinguishes file vs. directory symlinks and a `FileManifest` has no
/// manifest-driven way to know which the target is, so this only creates
/// file symlinks.
///
/// # Errors
/// Returns an error if the symlink cannot be created (creating symlinks on
/// Windows typically requires administrative privileges or Developer Mode).
pub fn create_symlink<P: AsRef<Path>>(path: P, target: P) -> Result<()> {
    std::os::windows::fs::symlink_file(target, path)?;

    Ok(())
}

/// No-op on Windows in every case: for a Unix-sourced entry, POSIX
/// permission bits have no ACL translation attempted here; for a
/// Windows-sourced entry (`source_os == SourceOs::Windows`), real
/// `FILE_ATTRIBUTE_*` restoration is a pre-existing gap this signature only
/// makes explicit — not implemented by this fix, tracked as a separate
/// future enhancement rather than folded into the source/target OS mismatch
/// this parameter exists to guard against (see the `unix.rs` counterpart).
///
/// # Errors
/// Never returns an error.
pub fn restore_permissions<P: AsRef<Path>>(
    _path: P,
    _mode: u32,
    _source_os: SourceOs,
) -> Result<()> {
    Ok(())
}

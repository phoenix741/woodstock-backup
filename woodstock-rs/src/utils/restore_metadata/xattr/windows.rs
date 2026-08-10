//! Stub implementation for extended attributes operations on platforms
//! without extended attribute support (Windows, or the `xattr` feature
//! disabled). Windows uses a different mechanism (alternate data streams)
//! for similar functionality, not implemented here.

use eyre::Result;
use std::path::Path;

use crate::FileManifestXAttr;

/// Always returns an empty vector — extended attribute reading is not
/// implemented for this build.
///
/// # Errors
/// Never returns an error.
pub fn read_xattr(_file: &Path) -> Result<Vec<FileManifestXAttr>> {
    Ok(Vec::new())
}

/// No-op — extended attribute restoration is not implemented for this
/// build.
///
/// # Errors
/// Never returns an error.
pub fn restore_xattr(_file: &Path, _xattrs: &[FileManifestXAttr]) -> Result<()> {
    Ok(())
}

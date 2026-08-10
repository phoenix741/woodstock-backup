//! Stub implementation for ACL operations on platforms without POSIX ACL
//! support (Windows, non-Linux Unix, or the `acl` feature disabled).

use eyre::Result;
use std::path::Path;

use crate::FileManifestAcl;

/// Always returns an empty vector — ACL reading is not implemented for this
/// build.
///
/// # Errors
/// Never returns an error.
pub fn read_acl(_file: &Path) -> Result<Vec<FileManifestAcl>> {
    Ok(Vec::new())
}

/// No-op — ACL restoration is not implemented for this build.
///
/// # Errors
/// Never returns an error.
pub fn restore_acl(_file: &Path, _acls: &[FileManifestAcl]) -> Result<()> {
    Ok(())
}

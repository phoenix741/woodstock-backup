/// Windows implementation for ACL operations.
///
/// This module provides stub implementations for ACL operations on Windows systems,
/// or when the 'acl' feature is disabled. On Windows, the implementations are empty
/// placeholders that perform no actual ACL operations.
use woodstock::FileManifestAcl;
use eyre::Result;
use std::path::Path;

/// Reads the Access Control Lists for a file on Windows.
///
/// This is a stub implementation that returns an empty vector,
/// as Windows ACL handling is not fully implemented.
///
/// # Arguments
/// * `_file` - Path to the file to read ACLs from. The parameter is unused in this implementation.
///
/// # Returns
/// * `Result<Vec<FileManifestAcl>>` - Always returns Ok with an empty vector.
pub fn read_acl(_file: &Path) -> Result<Vec<FileManifestAcl>> {
    Ok(Vec::new())
}

/// Restores Access Control Lists to a file on Windows.
///
/// This is a stub implementation that does nothing,
/// as Windows ACL handling is not fully implemented.
///
/// # Arguments
/// * `_file` - Path to the file to restore ACLs to. The parameter is unused in this implementation.
/// * `_acls` - Vector of ACL entries to restore. The parameter is unused in this implementation.
///
/// # Returns
/// * `Result<()>` - Always returns Ok.
pub fn restore_acl(_file: &Path, _acls: &[FileManifestAcl]) -> Result<()> {
    Ok(())
}

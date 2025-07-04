use eyre::Result;
use std::path::Path;
/// Windows implementation for extended attributes operations.
///
/// This module provides stub implementations for extended attributes operations
/// on Windows systems, or when the 'xattr' feature is disabled. On Windows,
/// the implementations are empty placeholders that perform no actual operations
/// since Windows uses a different mechanism (alternate data streams) for similar
/// functionality.
use woodstock::FileManifestXAttr;

/// Reads the extended attributes for a file on Windows.
///
/// This is a stub implementation that returns an empty vector,
/// as Windows extended attributes handling is not implemented.
///
/// # Arguments
/// * `_file` - Path to the file to read extended attributes from.
///   The parameter is unused in this implementation.
///
/// # Returns
/// * `Result<Vec<FileManifestXAttr>>` - Always returns Ok with an empty vector.
pub fn read_xattr(_file: &Path) -> Result<Vec<FileManifestXAttr>> {
    Ok(Vec::new())
}

/// Restores extended attributes to a file on Windows.
///
/// This is a stub implementation that does nothing,
/// as Windows extended attributes handling is not implemented.
///
/// # Arguments
/// * `_file` - Path to the file to restore extended attributes to.
///   The parameter is unused in this implementation.
/// * `_xattrs` - Vector of extended attribute entries to restore.
///   The parameter is unused in this implementation.
///
/// # Returns
/// * `Result<()>` - Always returns Ok.
pub fn restore_xattr(_file: &Path, _xattrs: &[FileManifestXAttr]) -> Result<()> {
    Ok(())
}

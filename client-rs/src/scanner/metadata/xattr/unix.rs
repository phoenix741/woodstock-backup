use eyre::Result;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::path::Path;
/// Unix implementation for extended attributes operations.
///
/// This module provides concrete implementations for reading and restoring
/// extended attributes on Unix systems when the 'xattr' feature is enabled.
/// It uses the `xattr` crate to interface with the underlying extended attributes system.
///
/// Extended attributes on Unix systems are name:value pairs associated with filesystem
/// objects (files, directories, symlinks, etc.) that can be used to store arbitrary
/// metadata not interpreted by the filesystem itself.
use woodstock::FileManifestXAttr;
use xattr::FileExt;

/// Reads the extended attributes for a file on Unix systems.
///
/// This function retrieves all extended attributes from a file and converts them
/// to the application's internal `FileManifestXAttr` format for storage.
///
/// # Arguments
/// * `file` - Path to the file to read extended attributes from.
///
/// # Returns
/// * `Result<Vec<FileManifestXAttr>>` - A vector of extended attribute entries if successful, or an error.
///
/// # Errors
/// Returns an error if:
/// * The file does not exist
/// * The process lacks permissions to read the file's extended attributes
/// * The underlying xattr functions fail
///
/// # Implementation Details
/// The function:
/// 1. Lists all extended attribute names using `xattr::list`
/// 2. For each attribute name, gets its value using `xattr::get`
/// 3. Converts both name and value to byte vectors for storage
pub fn read_xattr(file: &Path) -> Result<Vec<FileManifestXAttr>> {
    let attrs = xattr::list(file).map(|attrs| {
        attrs
            .filter_map(|attr| {
                xattr::get(file, &attr).ok()?.map(|value| {
                    let key = attr.as_encoded_bytes().to_vec();
                    FileManifestXAttr { key, value }
                })
            })
            .collect()
    })?;

    Ok(attrs)
}

/// Restores extended attributes to a file on Unix systems.
///
/// This function takes a list of `FileManifestXAttr` entries and applies them
/// to the specified file, restoring its extended attributes to match the saved state.
///
/// # Arguments
/// * `file` - Path to the file to restore extended attributes to.
/// * `xattrs` - Vector of extended attribute entries to restore to the file.
///
/// # Returns
/// * `Result<()>` - Success or error result.
///
/// # Errors
/// Returns an error if:
/// * The file does not exist
/// * The process lacks permissions to modify the file's extended attributes
/// * The underlying xattr functions fail
///
/// # Safety
/// This function uses `OsString::from_encoded_bytes_unchecked` which is unsafe
/// because it assumes that the byte sequence in the key is a valid encoding for
/// the platform. This is generally safe in this context because the keys were
/// originally obtained from the filesystem using the same encoding.
///
/// # Implementation Details
/// The function:
/// 1. Opens the target file for writing (required by FreeBSD's `extattr_set_fd`;
///    Linux's `fsetxattr` would also accept a read-only descriptor, but writing
///    is required for portability)
/// 2. For each extended attribute, converts its key from bytes to an `OsString`
/// 3. Sets the attribute on the file using the `FileExt` trait
pub fn restore_xattr(file: &Path, xattrs: &[FileManifestXAttr]) -> Result<()> {
    let file = OpenOptions::new().write(true).open(file)?;

    for xattr in xattrs {
        let key = unsafe { OsString::from_encoded_bytes_unchecked(xattr.key.clone()) };
        file.set_xattr(key, &xattr.value)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Round-trips a `user.*` extended attribute through `read_xattr` and
    /// `restore_xattr`. `user.*` is used because it requires no elevated
    /// privileges on either Linux or FreeBSD.
    #[test]
    fn test_read_and_restore_xattr_roundtrip() {
        let source = tempfile::NamedTempFile::new().unwrap();
        xattr::set(source.path(), "user.woodstock_test", b"hello").unwrap();

        let attrs = read_xattr(source.path()).unwrap();
        assert!(attrs
            .iter()
            .any(|a| a.key == b"user.woodstock_test" && a.value == b"hello"));

        let destination = tempfile::NamedTempFile::new().unwrap();
        restore_xattr(destination.path(), &attrs).unwrap();

        let restored = xattr::get(destination.path(), "user.woodstock_test").unwrap();
        assert_eq!(restored, Some(b"hello".to_vec()));
    }
}

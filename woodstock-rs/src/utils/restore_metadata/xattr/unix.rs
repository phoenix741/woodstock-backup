//! Unix implementation for extended attributes operations.
//!
//! Uses the `xattr` crate to interface with the underlying extended
//! attributes system. Extended attributes on Unix systems are name:value
//! pairs associated with filesystem objects (files, directories, symlinks,
//! etc.) that can be used to store arbitrary metadata not interpreted by
//! the filesystem itself.

use eyre::Result;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::path::Path;

use crate::FileManifestXAttr;
use xattr::FileExt;

/// Reads the extended attributes for a file on Unix systems.
///
/// # Errors
/// Returns an error if the file does not exist, the process lacks
/// permission to read its extended attributes, or the underlying `xattr`
/// calls fail.
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
/// # Errors
/// Returns an error if the file does not exist, the process lacks
/// permission to modify its extended attributes, or the underlying `xattr`
/// calls fail.
///
/// # Safety
/// Uses `OsString::from_encoded_bytes_unchecked` internally, which assumes
/// the byte sequence in each key is a valid encoding for the platform —
/// true here since the keys were originally obtained from the filesystem
/// using the same encoding.
///
/// # Implementation Details
/// Opens the target file for writing (required by FreeBSD's
/// `extattr_set_fd`; Linux's `fsetxattr` would also accept a read-only
/// descriptor, but writing is required for portability), then sets each
/// attribute via the `FileExt` trait.
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

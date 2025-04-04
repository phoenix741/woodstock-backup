use crate::FileManifestXAttr;
use eyre::Result;
use std::ffi::OsString;
use std::fs::File;
use std::path::Path;
use xattr::FileExt;

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

pub fn restore_xattr(file: &Path, xattrs: &[FileManifestXAttr]) -> Result<()> {
    let file = File::open(file)?;

    for xattr in xattrs {
        let key = unsafe { OsString::from_encoded_bytes_unchecked(xattr.key.clone()) };
        file.set_xattr(key, &xattr.value)?;
    }

    Ok(())
}

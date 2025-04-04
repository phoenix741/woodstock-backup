use crate::FileManifestXAttr;
use eyre::Result;
use std::path::Path;

pub fn read_xattr(_file: &Path) -> Result<Vec<FileManifestXAttr>> {
    Ok(Vec::new())
}

pub fn restore_xattr(_file: &Path, _xattrs: &[FileManifestXAttr]) -> Result<()> {
    Ok(())
}

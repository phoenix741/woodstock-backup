use crate::FileManifestAcl;
use eyre::Result;
use std::path::Path;

pub fn read_acl(_file: &Path) -> Result<Vec<FileManifestAcl>> {
    Ok(Vec::new())
}

pub fn restore_acl(_file: &Path, _acls: &[FileManifestAcl]) -> Result<()> {
    Ok(())
}

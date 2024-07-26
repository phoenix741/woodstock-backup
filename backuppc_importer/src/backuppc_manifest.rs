use woodstock::{manifest::PathManifest, FileManifest};

pub const BPC_DIGEST: &str = "backuppc_digest";

#[derive(Clone)]
pub struct FileManifestBackupPC {
    pub path: Vec<u8>,
    last_modified: i64,
    size: u64,
    pub backuppc_digest: Vec<u8>,
}

impl PathManifest for FileManifestBackupPC {
    fn array_path(&self) -> &Vec<u8> {
        &self.path
    }

    fn last_modified(&self) -> i64 {
        self.last_modified
    }

    fn size(&self) -> u64 {
        self.size
    }
}

impl From<FileManifest> for FileManifestBackupPC {
    fn from(manifest: FileManifest) -> Self {
        let backuppc_digest = manifest.metadata.get(BPC_DIGEST);
        let backuppc_digest = match backuppc_digest {
            Some(digest) => digest.clone(),
            None => Vec::<u8>::new(),
        };

        Self {
            last_modified: manifest.last_modified(),
            size: manifest.size(),
            path: manifest.path,
            backuppc_digest,
        }
    }
}

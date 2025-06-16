use woodstock::{manifest::PathManifest, FileManifest};

/// The key used in file metadata to store the `BackupPC` digest.
///
/// This constant is used to identify the digest value associated with a file in the `BackupPC` pool.
pub const BPC_DIGEST: &str = "backuppc_digest";

/// Represents a file manifest entry specific to `BackupPC`.
///
/// This struct is used to store essential information about a file in the `BackupPC` pool, including its path, last modification time, size, and digest.
///
/// # Fields
/// - `path`: The path to the file as a vector of bytes.
/// - `last_modified`: The last modification timestamp of the file (Unix timestamp).
/// - `size`: The size of the file in bytes.
/// - `backuppc_digest`: The digest value associated with the file in `BackupPC`.
#[derive(Clone)]
pub struct FileManifestBackupPC {
    /// The path to the file as a vector of bytes.
    pub path: Vec<u8>,
    /// The last modification timestamp of the file (Unix timestamp).
    last_modified: i64,
    /// The size of the file in bytes.
    size: u64,
    /// The digest value associated with the file in `BackupPC`.
    pub backuppc_digest: Vec<u8>,
}

impl PathManifest for FileManifestBackupPC {
    /// Returns the path to the file as a vector of bytes.
    fn array_path(&self) -> &Vec<u8> {
        &self.path
    }

    /// Returns the last modification timestamp of the file (Unix timestamp).
    fn last_modified(&self) -> i64 {
        self.last_modified
    }

    /// Returns the size of the file in bytes.
    fn size(&self) -> u64 {
        self.size
    }
}

impl From<FileManifest> for FileManifestBackupPC {
    /// Creates a `FileManifestBackupPC` from a generic `FileManifest`.
    ///
    /// # Arguments
    /// * `manifest` - The generic file manifest to convert.
    ///
    /// # Returns
    /// A `FileManifestBackupPC` containing the path, last modification time, size, and `BackupPC` digest.
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

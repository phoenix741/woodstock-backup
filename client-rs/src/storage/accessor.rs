use crate::scanner::{
    calculate_chunk_hash_future, get_files_with_hash, read_chunk, CreateManifestOptions,
};
use crate::storage::snapshots::{select_snapshot_manager, SnapshotCompletion, SnapshotReference};
use eyre::{eyre, Result};
use futures::{pin_mut, Stream, StreamExt};
use globset::GlobSet;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
use woodstock::{
    manifest::{IndexManifest, PathManifest},
    utils::path::{path_to_vec, vec_to_path},
    ChunkHashReply, ChunkHashRequest, ChunkInformation, FileChunk, FileManifestJournalEntry,
    ShareSnapshotResult, SnapshotMethod,
};

/// Represents a single path redirection mapping.
#[derive(Clone, Debug)]
pub struct PathRedirection {
    /// The original path that should be redirected
    pub origin_path: PathBuf,
    /// The alternative path where operations should actually be performed
    pub alternative_path: PathBuf,
}

impl PathRedirection {
    /// Creates a new path redirection mapping.
    pub fn new(origin_path: impl Into<PathBuf>, alternative_path: impl Into<PathBuf>) -> Self {
        Self {
            origin_path: origin_path.into(),
            alternative_path: alternative_path.into(),
        }
    }
}

/// Unified filesystem accessor that combines direct access, constraint validation, and path redirection.
///
/// This implementation provides all three functionalities in a single object:
/// - **Direct Access**: Basic filesystem operations (when no constraints or redirections)
/// - **Constraint Access**: Security restrictions to allowed directories (when allowed_directories is not empty)
/// - **Path Redirection**: Path mapping for snapshot-based backups (when redirections is not empty)
///
/// Configuration is done via the constructor parameters:
/// - Empty `allowed_directories` = all paths allowed
/// - Empty `redirections` = no path redirection
///
/// # Processing Order
/// 1. **Access Validation**: Check if the origin path is allowed if constraints are configured
/// 2. **Path Redirection**: Convert origin paths to alternative paths if configured
/// 3. **Operation Execution**: Execute the actual filesystem operation
/// 4. **Result Conversion**: Convert results back to origin paths if redirection was applied
pub struct FileSystemAccessor {
    /// Set of allowed directories for file access. Empty set means all paths are allowed.
    allowed_directories: HashSet<PathBuf>,
    /// List of path redirection mappings, ordered by specificity (most specific first)
    redirections: Vec<PathRedirection>,
    /// Active snapshots created by this accessor, stored for cleanup purposes
    active_snapshots: Vec<Box<dyn SnapshotReference>>,
}

impl FileSystemAccessor {
    /// Creates a new unified filesystem accessor with direct access only (no constraints or redirections).
    pub fn new() -> Self {
        Self {
            allowed_directories: HashSet::new(),
            redirections: vec![],
            active_snapshots: vec![],
        }
    }

    /// Creates a new unified filesystem accessor with constraint-based access control.
    ///
    /// # Arguments
    /// * `allowed_directories` - Vector of paths that are allowed for file access
    pub fn new_with_constraints(allowed_directories: &[PathBuf]) -> Self {
        let mut normalized_dirs = HashSet::new();

        for dir in allowed_directories {
            // Normalize paths by canonicalizing them if they exist
            if let Ok(canonical) = dir.canonicalize() {
                normalized_dirs.insert(canonical);
            } else {
                // If canonicalization fails, store the path as-is
                normalized_dirs.insert(dir.clone());
            }
        }

        Self {
            allowed_directories: normalized_dirs,
            redirections: vec![],
            active_snapshots: vec![],
        }
    }

    /// Creates a new unified filesystem accessor with path redirection support.
    ///
    /// # Arguments
    /// * `redirections` - Vector of path redirection mappings
    pub fn new_with_redirections(redirections: Vec<PathRedirection>) -> Self {
        // Sort redirections by path specificity (longer paths first)
        let mut sorted_redirections = redirections;
        sorted_redirections.sort_by(|a, b| {
            b.origin_path
                .as_os_str()
                .len()
                .cmp(&a.origin_path.as_os_str().len())
        });

        Self {
            allowed_directories: HashSet::new(),
            redirections: sorted_redirections,
            active_snapshots: vec![],
        }
    }

    /// Creates a new unified filesystem accessor with both constraints and redirections.
    ///
    /// # Arguments
    /// * `allowed_directories` - Vector of paths that are allowed for file access
    /// * `redirections` - Vector of path redirection mappings
    pub fn new_with_constraints_and_redirections(
        allowed_directories: &[PathBuf],
        redirections: Vec<PathRedirection>,
    ) -> Self {
        let mut normalized_dirs = HashSet::new();

        for dir in allowed_directories {
            if let Ok(canonical) = dir.canonicalize() {
                normalized_dirs.insert(canonical);
            } else {
                normalized_dirs.insert(dir.clone());
            }
        }

        // Sort redirections by path specificity (longer paths first)
        let mut sorted_redirections = redirections;
        sorted_redirections.sort_by(|a, b| {
            b.origin_path
                .as_os_str()
                .len()
                .cmp(&a.origin_path.as_os_str().len())
        });

        Self {
            allowed_directories: normalized_dirs,
            redirections: sorted_redirections,
            active_snapshots: vec![],
        }
    }

    /// Checks if a given path is within any of the allowed directories.
    /// Returns true if no constraints are configured (empty allowed_directories).
    ///
    /// # Arguments
    /// * `path` - The path to check
    ///
    /// # Returns
    /// `true` if the path is allowed, `false` otherwise
    fn is_path_allowed(&self, path: &Path) -> bool {
        // If no constraints are configured, allow all paths
        if self.allowed_directories.is_empty() {
            return true;
        }

        let normalized_path = match path.canonicalize() {
            Ok(canonical) => canonical,
            Err(_) => return false, // Path doesn't exist or can't be accessed
        };

        for allowed_dir in &self.allowed_directories {
            if normalized_path.starts_with(allowed_dir) {
                return true;
            }
        }

        false
    }

    /// Converts a path from the origin space to the alternative space.
    /// Returns the original path if no redirections are configured.
    ///
    /// # Arguments
    /// * `path` - The path to redirect
    ///
    /// # Returns
    /// The redirected path or the original path if no redirection applies
    fn redirect_path_to_alternative(&self, path: &Path) -> PathBuf {
        for redirection in &self.redirections {
            if let Ok(relative_path) = path.strip_prefix(&redirection.origin_path) {
                return redirection.alternative_path.join(relative_path);
            }
        }
        // If the path doesn't match any origin path, return it as-is
        path.to_path_buf()
    }

    /// Converts a path from the alternative space back to the origin space.
    /// Returns the original path if no redirections are configured.
    ///
    /// # Arguments
    /// * `path` - The path to convert back
    ///
    /// # Returns
    /// The origin path or the original path if no redirection applies
    fn redirect_path_to_origin(&self, path: &Path) -> PathBuf {
        for redirection in &self.redirections {
            if let Ok(relative_path) = path.strip_prefix(&redirection.alternative_path) {
                return redirection.origin_path.join(relative_path);
            }
        }
        // If the path doesn't match any alternative path, return it as-is
        path.to_path_buf()
    }

    /// Converts a ChunkHashRequest from origin space to alternative space (if redirection is configured)
    /// and validates access permissions.
    fn process_chunk_request(&self, request: ChunkHashRequest) -> Result<ChunkHashRequest> {
        // Step 1: Construct the full file path from the origin request for validation
        let share_path = Path::new(&request.share_path);
        let filename = vec_to_path(&request.filename);
        let origin_file_path = share_path.join(filename);

        // Step 2: Validate access permissions on the ORIGIN path
        if !self.is_path_allowed(&origin_file_path) {
            return Err(eyre!(
                "Unauthorized access to path: {}",
                origin_file_path.display()
            ));
        }

        // Step 3: Apply path redirection if configured
        let redirected_share_path = self.redirect_path_to_alternative(share_path);

        // Step 4: Return the processed request
        Ok(ChunkHashRequest {
            share_path: redirected_share_path.to_string_lossy().to_string(),
            filename: request.filename,
            algorithm: request.algorithm,
        })
    }

    /// Converts a ChunkInformation from origin space to alternative space (if redirection is configured)
    /// and validates access permissions.
    fn process_chunk_info(&self, chunk: &ChunkInformation) -> Result<ChunkInformation> {
        // Step 1: Construct the full file path from the origin request for validation
        let share_path = Path::new(&chunk.share_path);
        let filename = vec_to_path(&chunk.filename);
        let origin_file_path = share_path.join(filename);

        // Step 2: Validate access permissions on the ORIGIN path
        if !self.is_path_allowed(&origin_file_path) {
            return Err(eyre!(
                "Unauthorized access to path: {}",
                origin_file_path.display()
            ));
        }

        // Step 3: Apply path redirection if configured
        let redirected_share_path = self.redirect_path_to_alternative(share_path);

        // Step 4: Return the processed chunk info
        Ok(ChunkInformation {
            share_path: redirected_share_path.to_string_lossy().to_string(),
            filename: chunk.filename.clone(),
            chunks_id: chunk.chunks_id.clone(),
            algorithm: chunk.algorithm,
        })
    }

    /// Converts a FileManifestJournalEntry from alternative space back to origin space.
    /// This transforms the file paths in the manifest from the alternative (snapshot) path
    /// back to the original path that the caller expects.
    fn convert_journal_entry_to_origin(
        &self,
        mut entry: FileManifestJournalEntry,
    ) -> FileManifestJournalEntry {
        // Only apply conversion if redirections are configured
        if self.redirections.is_empty() {
            return entry;
        }

        if let Some(ref mut manifest) = entry.manifest {
            // Convert the path from bytes to PathBuf, redirect it, then convert back
            let path = manifest.path();
            let redirected_path = self.redirect_path_to_origin(&path);
            manifest.path = path_to_vec(redirected_path);
        }
        entry
    }

    /// Gets all redirection mappings.
    pub fn get_redirections(&self) -> &[PathRedirection] {
        &self.redirections
    }

    /// Gets allowed directories.
    pub fn get_allowed_directories(&self) -> &HashSet<PathBuf> {
        &self.allowed_directories
    }

    /// Checks if constraints are enabled (non-empty allowed directories).
    pub fn has_constraints(&self) -> bool {
        !self.allowed_directories.is_empty()
    }

    /// Checks if redirections are enabled (non-empty redirections list).
    pub fn has_redirections(&self) -> bool {
        !self.redirections.is_empty()
    }

    /// Adds a share path with automatic snapshot creation if available.
    ///
    /// This method attempts to create a snapshot of the given share path using
    /// the best available snapshot manager. If successful, it adds a path redirection
    /// from the original path to the snapshot path.
    ///
    /// # Arguments
    /// * `share_path` - The path to create a snapshot for and add redirection
    ///
    /// # Returns
    /// * `Ok(ShareSnapshotResult)` describing which snapshot method was used and any failure reason.
    ///
    /// # Example
    /// ```no_run
    /// use woodstock_client_rs::storage::accessor::FileSystemAccessor;
    /// # async fn example() -> eyre::Result<()> {
    /// let mut accessor = FileSystemAccessor::new();
    /// accessor.add_share_path("/home/user/documents").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_share_path<P: AsRef<Path>>(
        &mut self,
        share_path: P,
    ) -> Result<ShareSnapshotResult> {
        let share_path = share_path.as_ref();

        // Try to find an available snapshot manager for this path
        if let Some(snapshot_manager) = select_snapshot_manager(share_path).await {
            let method = snapshot_manager.snapshot_method() as i32;
            // Create a snapshot
            match snapshot_manager.create_snapshot(share_path).await {
                Ok(snapshot_ref) => {
                    let snapshot_path = snapshot_ref.path().to_path_buf();

                    // Create a path redirection from the original path to the snapshot
                    let redirection = PathRedirection::new(share_path, snapshot_path);

                    // Add the redirection to our list, maintaining the sort order (most specific first)
                    self.redirections.push(redirection);
                    self.redirections.sort_by(|a, b| {
                        b.origin_path
                            .as_os_str()
                            .len()
                            .cmp(&a.origin_path.as_os_str().len())
                    });

                    // Store the snapshot reference for cleanup later
                    self.active_snapshots.push(snapshot_ref);

                    tracing::info!(
                        "Created snapshot for share path '{}' using {} snapshot manager",
                        share_path.display(),
                        snapshot_manager.manager_name()
                    );

                    Ok(ShareSnapshotResult {
                        method,
                        failure_reason: None,
                    })
                }
                Err(err) => {
                    tracing::error!(
                        "Failed to create snapshot for share path '{}' using {} manager: {}",
                        share_path.display(),
                        snapshot_manager.manager_name(),
                        err
                    );
                    Ok(ShareSnapshotResult {
                        method: SnapshotMethod::None as i32,
                        failure_reason: Some(err.to_string()),
                    })
                }
            }
        } else {
            tracing::debug!(
                "No snapshot manager available for share path '{}'",
                share_path.display()
            );
            Ok(ShareSnapshotResult {
                method: SnapshotMethod::None as i32,
                failure_reason: None,
            })
        }
    }

    /// Gets the list of active snapshots managed by this accessor.
    pub fn get_active_snapshots(&self) -> &[Box<dyn SnapshotReference>] {
        &self.active_snapshots
    }

    /// Cleans up all active snapshots and removes their associated redirections.
    ///
    /// This method will attempt to delete all snapshots that were created by this
    /// accessor and remove their corresponding path redirections. After calling this
    /// method, the accessor will behave as if no snapshots were ever created.
    ///
    /// # Returns
    /// * `Ok(())` if all snapshots were cleaned up successfully
    /// * `Err(...)` if any snapshot deletion failed (partial cleanup may have occurred)
    ///
    /// Clean up all active snapshots by calling their self-deletion methods
    ///
    /// This method removes all active snapshots by calling `delete_self()` on each
    /// snapshot reference. This approach is robust and reliable because:
    ///
    /// - No snapshot manager re-detection is required
    /// - Each snapshot contains the exact information needed for deletion
    /// - Works regardless of filesystem changes or manager availability
    ///
    /// After successful deletion, the corresponding path redirections are also
    /// removed from the accessor.
    ///
    /// # Returns
    /// * `Ok(())` if all snapshots were cleaned up successfully
    /// * `Err(...)` if any snapshot deletion failed (partial cleanup may have occurred)
    ///
    /// # Note
    /// Each snapshot reference contains the information needed to delete itself,
    /// eliminating the need to re-detect snapshot managers.
    pub async fn cleanup_all_snapshots(&mut self) -> Result<()> {
        self.cleanup_all_snapshots_with_completion(SnapshotCompletion::Abort)
            .await
    }

    /// Cleans up all active snapshots after a successful backup.
    pub async fn cleanup_all_snapshots_success(&mut self) -> Result<()> {
        self.cleanup_all_snapshots_with_completion(SnapshotCompletion::Success)
            .await
    }

    async fn cleanup_all_snapshots_with_completion(
        &mut self,
        completion: SnapshotCompletion,
    ) -> Result<()> {
        let mut errors = Vec::new();

        // Collect all snapshot paths before cleanup to identify redirections to remove
        let snapshot_paths: Vec<PathBuf> = self
            .active_snapshots
            .iter()
            .map(|snapshot| snapshot.path().to_path_buf())
            .collect();

        // Attempt to delete each snapshot using its own delete_self method
        for snapshot in self.active_snapshots.drain(..) {
            let snapshot_path = snapshot.path();

            if let Err(e) = snapshot.finalize_self(completion).await {
                tracing::error!(
                    "Failed to finalize snapshot '{}' with outcome {:?}: {}",
                    snapshot_path.display(),
                    completion,
                    e
                );
                errors.push(e);
            } else {
                tracing::info!(
                    "Successfully finalized snapshot '{}' with outcome {:?}",
                    snapshot_path.display(),
                    completion
                );
            }
        }

        // Remove redirections that point to the deleted snapshots
        self.redirections
            .retain(|redirection| !snapshot_paths.contains(&redirection.alternative_path));

        // Return error if any deletions failed
        if !errors.is_empty() {
            return Err(eyre!(
                "Failed to delete {} snapshot(s): {}",
                errors.len(),
                errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        Ok(())
    }

    /// Creates a clone of this accessor without the active snapshots.
    ///
    /// This method creates a new `FileSystemAccessor` with the same configuration
    /// (allowed directories and redirections) but without any active snapshots.
    /// This is useful when you need to share the accessor configuration across
    /// different contexts without sharing the snapshot lifecycle.
    ///
    /// # Returns
    /// A new `FileSystemAccessor` with the same configuration but no active snapshots
    pub fn clone_without_snapshots(&self) -> Self {
        Self {
            allowed_directories: self.allowed_directories.clone(),
            redirections: self.redirections.clone(),
            active_snapshots: vec![],
        }
    }
}

impl Default for FileSystemAccessor {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystemAccessor {
    /// Gets files with hash for backup synchronization.
    /// Applies path redirection and access constraints as configured.
    pub fn get_files_with_hash<'a, P: Into<PathBuf>, T: PathManifest>(
        &self,
        index: &'a mut IndexManifest<T>,
        share_path: P,
        includes: &'a GlobSet,
        excludes: &'a GlobSet,
        options: &'a CreateManifestOptions,
    ) -> impl Stream<Item = FileManifestJournalEntry> + 'a {
        let self_clone = self.clone_without_snapshots();
        let share_path = share_path.into();

        async_stream::stream!({
            // Step 1: Validate access to the ORIGIN share path
            if !self_clone.is_path_allowed(&share_path) {
                // Yield an error entry or simply return (skip processing)
                return;
            }

            // Step 2: Apply path redirection if configured
            let redirected_share_path = self_clone.redirect_path_to_alternative(&share_path);

            // Step 3: Get files using the base implementation
            let stream =
                get_files_with_hash(index, redirected_share_path, includes, excludes, options);
            pin_mut!(stream);

            // Step 4: Process each entry, converting paths back to origin space if needed
            while let Some(entry) = stream.next().await {
                let converted_entry = self_clone.convert_journal_entry_to_origin(entry);
                yield converted_entry;
            }
        })
    }

    /// Calculates chunk hash for a file.
    /// Applies path redirection and access constraints as configured.
    pub async fn calculate_chunk_hash_future(
        &self,
        chunk: ChunkHashRequest,
    ) -> Result<ChunkHashReply> {
        // Process the chunk request (apply redirection and validate access)
        let processed_chunk = self.process_chunk_request(chunk)?;

        // Execute the actual hash calculation
        calculate_chunk_hash_future(&processed_chunk).await
    }

    /// Reads chunks from a file.
    /// Applies path redirection and access constraints as configured.
    pub fn read_chunk<'a, 'b>(
        &'a self,
        chunk: ChunkInformation,
    ) -> impl Stream<Item = Result<FileChunk, std::io::Error>> + 'b {
        let self_clone = self.clone_without_snapshots();

        async_stream::stream!({
            // Process the chunk information (apply redirection and validate access)
            match self_clone.process_chunk_info(&chunk) {
                Ok(processed_chunk) => {
                    // Execute the actual chunk reading
                    let stream = read_chunk(processed_chunk);
                    pin_mut!(stream);

                    while let Some(chunk_result) = stream.next().await {
                        yield chunk_result;
                    }
                }
                Err(e) => {
                    // Convert the eyre error to std::io::Error
                    let io_error = std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        format!("Unauthorized access: {}", e),
                    );
                    yield Err(io_error);
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "linux")]
    use crate::storage::snapshots::btrfs;
    use futures::{pin_mut, StreamExt};
    use std::fs;
    use tempfile::TempDir;
    use woodstock::{EntryState, EntryType, FileManifest};

    #[test]
    fn test_unified_accessor_creation() {
        // Test default creation (no constraints, no redirections)
        let accessor = FileSystemAccessor::new();
        assert!(!accessor.has_constraints());
        assert!(!accessor.has_redirections());

        // Test with constraints only
        let allowed_dirs = vec![PathBuf::from("/home"), PathBuf::from("/etc")];
        let accessor = FileSystemAccessor::new_with_constraints(&allowed_dirs);
        assert!(accessor.has_constraints());
        assert!(!accessor.has_redirections());

        // Test with redirections only
        let redirections = vec![PathRedirection::new("/home", "/home/.snapshots/daily")];
        let accessor = FileSystemAccessor::new_with_redirections(redirections);
        assert!(!accessor.has_constraints());
        assert!(accessor.has_redirections());

        // Test with both constraints and redirections
        let accessor = FileSystemAccessor::new_with_constraints_and_redirections(
            &allowed_dirs,
            vec![PathRedirection::new("/home", "/home/.snapshots/daily")],
        );
        assert!(accessor.has_constraints());
        assert!(accessor.has_redirections());
    }

    #[test]
    fn test_path_constraint_validation() {
        let temp_dir = TempDir::new().unwrap();
        let allowed_path = temp_dir.path().join("allowed");
        let forbidden_path = temp_dir.path().join("forbidden");
        let allowed_subdir = allowed_path.join("subdir");

        fs::create_dir_all(&allowed_path).unwrap();
        fs::create_dir_all(&forbidden_path).unwrap();
        fs::create_dir_all(&allowed_subdir).unwrap();

        let accessor = FileSystemAccessor::new_with_constraints(&[allowed_path.clone()]);

        // Test allowed paths
        assert!(accessor.is_path_allowed(&allowed_path));
        assert!(accessor.is_path_allowed(&allowed_subdir));

        // Test forbidden paths
        assert!(!accessor.is_path_allowed(&forbidden_path));
    }

    #[test]
    fn test_no_constraints_allows_all() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("test");
        fs::create_dir_all(&test_path).unwrap();

        // No constraints = all paths allowed
        let accessor = FileSystemAccessor::new();
        assert!(accessor.is_path_allowed(&test_path));
        assert!(accessor.is_path_allowed(&temp_dir.path()));
    }

    #[test]
    fn test_path_redirection() {
        let accessor = FileSystemAccessor::new_with_redirections(vec![PathRedirection::new(
            "/home",
            "/home/.snapshots/daily",
        )]);

        // Test redirection to alternative
        let origin_path = Path::new("/home/user/documents");
        let redirected = accessor.redirect_path_to_alternative(origin_path);
        assert_eq!(
            redirected,
            PathBuf::from("/home/.snapshots/daily/user/documents")
        );

        // Test redirection back to origin
        let alt_path = Path::new("/home/.snapshots/daily/user/documents");
        let back_to_origin = accessor.redirect_path_to_origin(alt_path);
        assert_eq!(back_to_origin, PathBuf::from("/home/user/documents"));

        // Test non-matching path (should remain unchanged)
        let unrelated_path = Path::new("/etc/config");
        let unchanged = accessor.redirect_path_to_alternative(unrelated_path);
        assert_eq!(unchanged, PathBuf::from("/etc/config"));
    }

    #[test]
    fn test_multiple_redirections_specificity() {
        let accessor = FileSystemAccessor::new_with_redirections(vec![
            PathRedirection::new("/", "/.snapshots/root"),
            PathRedirection::new("/home", "/home/.snapshots/daily"),
            PathRedirection::new("/home/user", "/home/user/.local/snapshots"),
        ]);

        // Most specific should win
        let path = Path::new("/home/user/documents/file.txt");
        let redirected = accessor.redirect_path_to_alternative(path);
        assert_eq!(
            redirected,
            PathBuf::from("/home/user/.local/snapshots/documents/file.txt")
        );

        // Second level specificity
        let path = Path::new("/home/other/file.txt");
        let redirected = accessor.redirect_path_to_alternative(path);
        assert_eq!(
            redirected,
            PathBuf::from("/home/.snapshots/daily/other/file.txt")
        );

        // Root level redirection
        let path = Path::new("/etc/config");
        let redirected = accessor.redirect_path_to_alternative(path);
        assert_eq!(redirected, PathBuf::from("/.snapshots/root/etc/config"));
    }

    #[tokio::test]
    async fn test_calculate_chunk_hash_with_constraints() {
        let temp_dir = TempDir::new().unwrap();
        let allowed_path = temp_dir.path().join("allowed");
        let forbidden_path = temp_dir.path().join("forbidden");
        let test_file = allowed_path.join("test.txt");

        fs::create_dir_all(&allowed_path).unwrap();
        fs::create_dir_all(&forbidden_path).unwrap();
        fs::write(&test_file, b"test content").unwrap();

        let accessor = FileSystemAccessor::new_with_constraints(&[allowed_path.clone()]);

        // Test allowed access
        let allowed_request = ChunkHashRequest {
            share_path: allowed_path.to_string_lossy().to_string(),
            filename: b"test.txt".to_vec(),
            algorithm: 0,
        };
        let result = accessor.calculate_chunk_hash_future(allowed_request).await;
        assert!(result.is_ok());

        // Test forbidden access
        let forbidden_request = ChunkHashRequest {
            share_path: forbidden_path.to_string_lossy().to_string(),
            filename: b"test.txt".to_vec(),
            algorithm: 0,
        };
        let result = accessor
            .calculate_chunk_hash_future(forbidden_request)
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unauthorized access"));
    }

    #[tokio::test]
    async fn test_calculate_chunk_hash_with_redirection() {
        let temp_dir = TempDir::new().unwrap();
        let origin_path = temp_dir.path().join("origin");
        let snapshot_path = temp_dir.path().join("snapshot");
        let test_file = snapshot_path.join("test.txt");

        fs::create_dir_all(&origin_path).unwrap();
        fs::create_dir_all(&snapshot_path).unwrap();
        fs::write(&test_file, b"snapshot content").unwrap();

        let accessor = FileSystemAccessor::new_with_redirections(vec![PathRedirection::new(
            &origin_path,
            &snapshot_path,
        )]);

        // Request using origin path should be redirected to snapshot
        let request = ChunkHashRequest {
            share_path: origin_path.to_string_lossy().to_string(),
            filename: b"test.txt".to_vec(),
            algorithm: 0,
        };
        let result = accessor.calculate_chunk_hash_future(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_read_chunk_with_constraints_and_redirection() {
        let temp_dir = TempDir::new().unwrap();
        let origin_path = temp_dir.path().join("data");
        let snapshot_path = temp_dir.path().join("snapshots").join("data");
        let origin_file = origin_path.join("file.txt"); // Create file in origin too
        let snapshot_file = snapshot_path.join("file.txt");

        fs::create_dir_all(&origin_path).unwrap();
        fs::create_dir_all(&snapshot_path).unwrap();
        fs::write(&origin_file, b"origin content").unwrap(); // Create in origin for path validation
        fs::write(&snapshot_file, b"snapshot content").unwrap(); // Actual content in snapshot

        // Allow the ORIGIN path since constraints are checked on origin paths
        let accessor = FileSystemAccessor::new_with_constraints_and_redirections(
            &[origin_path.clone()], // Allow origin path for validation
            vec![PathRedirection::new(&origin_path, &snapshot_path)],
        );

        let chunk_info = ChunkInformation {
            share_path: origin_path.to_string_lossy().to_string(),
            filename: b"file.txt".to_vec(),
            chunks_id: vec![],
            algorithm: 0,
        };

        let stream = accessor.read_chunk(chunk_info);
        pin_mut!(stream);

        let mut chunk_count = 0;
        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(_) => chunk_count += 1,
                Err(e) => {
                    // Should not get permission errors with our setup
                    if e.kind() == std::io::ErrorKind::PermissionDenied {
                        panic!("Unexpected permission error: {}", e);
                    }
                    // Other errors are acceptable (file format, etc.)
                    break;
                }
            }
        }

        // We should have received at least one chunk (indicating redirection worked)
        assert!(chunk_count > 0);
    }

    #[tokio::test]
    async fn test_read_chunk_unauthorized_access() {
        let temp_dir = TempDir::new().unwrap();
        let allowed_path = temp_dir.path().join("allowed");
        let forbidden_path = temp_dir.path().join("forbidden");

        fs::create_dir_all(&allowed_path).unwrap();
        fs::create_dir_all(&forbidden_path).unwrap();

        let accessor = FileSystemAccessor::new_with_constraints(&[allowed_path]);

        let chunk_info = ChunkInformation {
            share_path: forbidden_path.to_string_lossy().to_string(),
            filename: b"test.txt".to_vec(),
            chunks_id: vec![],
            algorithm: 0,
        };

        let stream = accessor.read_chunk(chunk_info);
        pin_mut!(stream);

        // Should get permission denied error
        let result = stream.next().await;
        assert!(result.is_some());
        let chunk_result = result.unwrap();
        assert!(chunk_result.is_err());
        let error = chunk_result.unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
        assert!(error.to_string().contains("Unauthorized access"));
    }

    #[test]
    fn test_journal_entry_conversion() {
        let accessor = FileSystemAccessor::new_with_redirections(vec![PathRedirection::new(
            "/data",
            "/data/.snapshots/daily",
        )]);

        let snapshot_path = "/data/.snapshots/daily/documents/report.pdf";
        let journal_entry = FileManifestJournalEntry {
            entry_type: EntryType::Add as i32,
            manifest: Some(FileManifest {
                path: snapshot_path.as_bytes().to_vec(),
                ..Default::default()
            }),
            state: EntryState::Metadata as i32,
            ..Default::default()
        };

        let converted = accessor.convert_journal_entry_to_origin(journal_entry);
        if let Some(manifest) = converted.manifest {
            let converted_path = String::from_utf8(manifest.path).expect("Invalid UTF-8");
            assert_eq!(converted_path, "/data/documents/report.pdf");
        } else {
            panic!("Manifest should be present after conversion");
        }
    }

    #[test]
    fn test_journal_entry_no_redirection() {
        let accessor = FileSystemAccessor::new(); // No redirections

        let original_path = "/data/documents/report.pdf";
        let journal_entry = FileManifestJournalEntry {
            entry_type: EntryType::Add as i32,
            manifest: Some(FileManifest {
                path: original_path.as_bytes().to_vec(),
                ..Default::default()
            }),
            state: EntryState::Metadata as i32,
            ..Default::default()
        };

        let converted = accessor.convert_journal_entry_to_origin(journal_entry);
        if let Some(manifest) = converted.manifest {
            let converted_path = String::from_utf8(manifest.path).expect("Invalid UTF-8");
            assert_eq!(converted_path, original_path); // Should remain unchanged
        }
    }

    #[test]
    fn test_accessor_getters() {
        let allowed_dirs = vec![PathBuf::from("/home"), PathBuf::from("/data")];
        let redirections = vec![
            PathRedirection::new("/home", "/home/.snapshots/daily"),
            PathRedirection::new("/data", "/data/.backups/hourly"),
        ];

        let accessor = FileSystemAccessor::new_with_constraints_and_redirections(
            &allowed_dirs,
            redirections.clone(),
        );

        // Test getters
        assert_eq!(accessor.get_redirections().len(), 2);
        assert!(accessor.has_constraints());
        assert!(accessor.has_redirections());

        // Verify allowed directories (might be canonicalized)
        let stored_dirs = accessor.get_allowed_directories();
        assert!(!stored_dirs.is_empty());
    }

    #[test]
    fn test_nonexistent_path_constraints() {
        let temp_dir = TempDir::new().unwrap();
        let existing_path = temp_dir.path().join("existing");
        let nonexistent_path = temp_dir.path().join("nonexistent");

        fs::create_dir_all(&existing_path).unwrap();
        // Don't create nonexistent_path

        let accessor = FileSystemAccessor::new_with_constraints(&[existing_path.clone()]);

        // Existing path should be allowed
        assert!(accessor.is_path_allowed(&existing_path));

        // Nonexistent path should be rejected (canonicalize fails)
        assert!(!accessor.is_path_allowed(&nonexistent_path));
        assert!(!accessor.is_path_allowed(&existing_path.join("does_not_exist")));
    }

    #[test]
    fn test_edge_case_empty_configurations() {
        // Test with empty redirections but non-empty constraints
        let temp_dir = TempDir::new().unwrap();
        let allowed_path = temp_dir.path().join("allowed");
        fs::create_dir_all(&allowed_path).unwrap();

        let accessor = FileSystemAccessor::new_with_constraints_and_redirections(
            &[allowed_path.clone()],
            vec![], // Empty redirections
        );

        assert!(accessor.has_constraints());
        assert!(!accessor.has_redirections());
        assert!(accessor.is_path_allowed(&allowed_path));

        // Test with empty constraints but non-empty redirections
        let accessor = FileSystemAccessor::new_with_constraints_and_redirections(
            &[], // Empty constraints
            vec![PathRedirection::new("/home", "/home/.snapshots")],
        );

        assert!(!accessor.has_constraints());
        assert!(accessor.has_redirections());
        assert!(accessor.is_path_allowed(&allowed_path)); // All allowed with empty constraints
    }

    #[test]
    fn test_complex_multi_level_redirections() {
        let accessor = FileSystemAccessor::new_with_redirections(vec![
            PathRedirection::new("/app/data/critical", "/app/data/critical/.realtime"),
            PathRedirection::new("/app/data", "/app/data/.hourly"),
            PathRedirection::new("/app", "/app/.daily"),
        ]);

        // Test that most specific redirection wins
        let critical_path = Path::new("/app/data/critical/file.db");
        let redirected = accessor.redirect_path_to_alternative(critical_path);
        assert_eq!(
            redirected,
            PathBuf::from("/app/data/critical/.realtime/file.db")
        );

        // Test second level specificity
        let data_path = Path::new("/app/data/general/file.dat");
        let redirected = accessor.redirect_path_to_alternative(data_path);
        assert_eq!(
            redirected,
            PathBuf::from("/app/data/.hourly/general/file.dat")
        );

        // Test general app level
        let app_path = Path::new("/app/config/settings.conf");
        let redirected = accessor.redirect_path_to_alternative(app_path);
        assert_eq!(
            redirected,
            PathBuf::from("/app/.daily/config/settings.conf")
        );
    }

    #[tokio::test]
    async fn test_constraints_checked_before_redirection() {
        let temp_dir = TempDir::new().unwrap();
        let forbidden_origin = temp_dir.path().join("forbidden");
        let allowed_snapshot = temp_dir.path().join("snapshots").join("allowed");
        let test_file = allowed_snapshot.join("file.txt");

        fs::create_dir_all(&forbidden_origin).unwrap();
        fs::create_dir_all(&allowed_snapshot).unwrap();
        fs::write(&test_file, b"test content").unwrap();

        // Create accessor that:
        // - FORBIDS access to the origin path
        // - BUT the redirection would point to an allowed location
        let accessor = FileSystemAccessor::new_with_constraints_and_redirections(
            &[allowed_snapshot.clone()], // Only allow snapshot path, NOT origin
            vec![PathRedirection::new(&forbidden_origin, &allowed_snapshot)],
        );

        // Request access to forbidden origin path
        let request = ChunkHashRequest {
            share_path: forbidden_origin.to_string_lossy().to_string(),
            filename: b"file.txt".to_vec(),
            algorithm: 0,
        };

        // Should fail because constraints are checked on ORIGIN path BEFORE redirection
        let result = accessor.calculate_chunk_hash_future(request).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unauthorized access"));

        // Also test with read_chunk
        let chunk_info = ChunkInformation {
            share_path: forbidden_origin.to_string_lossy().to_string(),
            filename: b"file.txt".to_vec(),
            chunks_id: vec![],
            algorithm: 0,
        };

        let stream = accessor.read_chunk(chunk_info);
        pin_mut!(stream);

        // Should get permission denied error immediately
        let result = stream.next().await;
        assert!(result.is_some());
        let chunk_result = result.unwrap();
        assert!(chunk_result.is_err());
        let error = chunk_result.unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
        assert!(error.to_string().contains("Unauthorized access"));
    }

    #[test]
    #[cfg(unix)]
    fn test_btrfs_snapshot_reference() {
        let redirection_path = PathBuf::from("/test/snapshot/path");
        let snapshot_root_path = PathBuf::from("/test/snapshot");
        let reference = btrfs::BtrfsSnapshotReference::new(
            redirection_path.clone(),
            snapshot_root_path.clone(),
            false, // sudo not required for tests
        );

        assert_eq!(reference.path(), redirection_path.as_path());
        assert_eq!(reference.as_string(), "/test/snapshot/path");
        assert_eq!(reference.snapshot_root_path(), snapshot_root_path.as_path());
    }

    #[test]
    fn test_clone_without_snapshots() {
        let temp_dir = TempDir::new().unwrap();
        let allowed_path = temp_dir.path().join("allowed");
        fs::create_dir_all(&allowed_path).unwrap();

        let redirections = vec![PathRedirection::new("/home", "/home/.snapshots/daily")];
        let accessor = FileSystemAccessor::new_with_constraints_and_redirections(
            &[allowed_path.clone()],
            redirections,
        );

        let cloned = accessor.clone_without_snapshots();

        // Should have same configuration
        assert!(cloned.has_constraints());
        assert!(cloned.has_redirections());
        assert_eq!(cloned.get_redirections().len(), 1);
        assert_eq!(cloned.get_allowed_directories().len(), 1);

        // But no active snapshots
        assert_eq!(cloned.get_active_snapshots().len(), 0);
    }

    #[tokio::test]
    async fn test_add_share_path_no_snapshot_manager() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("test");
        fs::create_dir_all(&test_path).unwrap();

        let mut accessor = FileSystemAccessor::new();

        // Should not fail even if no snapshot manager is available
        let result = accessor.add_share_path(&test_path).await;
        assert!(result.is_ok());

        // Should have no redirections or snapshots
        assert_eq!(accessor.get_redirections().len(), 0);
        assert_eq!(accessor.get_active_snapshots().len(), 0);
    }

    #[tokio::test]
    async fn test_cleanup_all_snapshots_empty() {
        let mut accessor = FileSystemAccessor::new();

        // Should not fail when there are no snapshots
        let result = accessor.cleanup_all_snapshots().await;
        assert!(result.is_ok());
    }
}

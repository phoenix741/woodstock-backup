use eyre::Result;
use fs4::fs_std::FileExt;
use log::{debug, error, warn};
use prost::Message;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use uuid::Uuid;
// Add new imports for atomic operations
use fs4::tokio::AsyncFileExt;

/// Type of lock for the pool
#[derive(Clone, Copy, PartialEq, Debug, ::prost::Enumeration)]
pub enum LockType {
    /// Shared lock, allows concurrent compatible operations (backup, deletion, compaction)
    Shared = 0,
    /// Exclusive lock, prevents any other operation (cleaning, integrity verification)
    Exclusive = 1,
}

/// Represents an individual lock entry
#[derive(Clone, PartialEq, ::prost::Message)]
struct LockEntry {
    /// Process ID that owns the lock
    #[prost(uint64, tag = "1")]
    pub pid: u64,
    /// Timestamp when the lock was created or last updated
    #[prost(uint64, tag = "2")]
    pub timestamp: u64,
    /// Name identifier for the lock
    #[prost(string, tag = "3")]
    pub lock_name: String,
    /// Type of lock (shared or exclusive)
    #[prost(enumeration = "LockType", tag = "4")]
    pub lock_type: i32,
    /// Unique identifier for this specific lock instance
    #[prost(string, tag = "5")]
    pub lock_uuid: String,
}

/// Represents the data stored in a lock file.
#[derive(Clone, PartialEq, ::prost::Message)]
struct LockFileData {
    /// List of active locks
    #[prost(message, repeated, tag = "1")]
    pub locks: Vec<LockEntry>,
}

/// Interval in seconds between lock file status checks
const CHECK_INTERVAL: f64 = 5.0; // seconds
/// Interval in seconds between lock file timestamp updates
const UPDATE_INTERVAL: f64 = 30.0; // seconds
/// Maximum time in seconds to wait for a lock to be released
const MAX_WAIT_TIME: u64 = 3_600; // seconds

// Note: Stale lock detection occurs after 3 * UPDATE_INTERVAL = 90 seconds
// This is intentionally long to avoid false positives from temporary system slowdowns

/// Represents a lock for a resource, such as a backup pool.
///
/// The lock ensures that only one process can access the resource at a time.
///
/// The resource that can be locked is the pool. The pool can be locked for the following reasons:
/// The lock ensures that only one process can access the resource at a time.
/// - when the refcnt is updated (write lock)
/// - when unused file are removed (write lock)
/// - when a file is read (read lock)
/// - when the refcnt is read (read lock)
/// - when the refcnt is checked (read lock)
/// - when a file is added (added lock)
pub struct PoolLock {
    /// Name identifier for the lock
    name: String,
    /// Unique identifier for this lock instance
    uuid: Uuid,
    /// Path to the lock file
    lock_file: PathBuf,
    /// Flag indicating whether the lock is currently held
    locked: bool,
    /// Handle for the background task that updates the lock file timestamp
    abort_handle: Option<tokio::task::AbortHandle>,
}

impl PoolLock {
    #[must_use]
    pub fn new_with_filename<P: AsRef<Path>>(path: &P, name: &str) -> Self {
        let path = path.as_ref();
        PoolLock {
            name: name.to_string(),
            uuid: Uuid::new_v4(),
            lock_file: path.to_path_buf(),
            locked: false,
            abort_handle: None,
        }
    }

    #[must_use]
    pub fn new_with_name<P: AsRef<Path>>(path: &P, name: &str) -> Self {
        let path = path.as_ref();
        PoolLock {
            name: name.to_string(),
            uuid: Uuid::new_v4(),
            lock_file: path.join("lock"),
            locked: false,
            abort_handle: None,
        }
    }

    #[must_use]
    pub fn new<P: AsRef<Path>>(path: &P) -> Self {
        let path = path.as_ref();
        PoolLock {
            name: path
                .to_str()
                .map(std::string::ToString::to_string)
                .unwrap_or_default(),
            uuid: Uuid::new_v4(),
            lock_file: path.join("lock"),
            locked: false,
            abort_handle: None,
        }
    }

    /// Acquires a lock on the pool.
    /// For backwards compatibility, this acquires an exclusive lock.
    ///
    /// # Returns
    ///
    /// Returns a result containing the locked pool lock instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the lock cannot be acquired after waiting for the maximum time.
    pub async fn lock(self) -> Result<Self> {
        self.lock_internal(LockType::Exclusive).await
    }

    /// Acquires a shared lock on the pool.
    /// Shared locks allow concurrent compatible operations (backup, deletion, compaction).
    ///
    /// # Returns
    ///
    /// Returns a result containing the locked pool lock instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the lock cannot be acquired after waiting for the maximum time.
    pub async fn lock_shared(self) -> Result<Self> {
        self.lock_internal(LockType::Shared).await
    }

    /// Acquires an exclusive lock on the pool.
    /// Exclusive locks prevent any other operation (cleaning, integrity verification).
    ///
    /// # Returns
    ///
    /// Returns a result containing the locked pool lock instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the lock cannot be acquired after waiting for the maximum time.
    pub async fn lock_exclusive(self) -> Result<Self> {
        self.lock_internal(LockType::Exclusive).await
    }

    /// Internal implementation for acquiring locks on the pool.
    ///
    /// # Arguments
    ///
    /// * `lock_type` - The type of lock to acquire (shared or exclusive)
    ///
    /// # Returns
    ///
    /// Returns a result containing the locked pool lock instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the lock cannot be acquired after waiting for the maximum time.
    async fn lock_internal(mut self, lock_type: LockType) -> Result<Self> {
        // Add an epsilon value to check_interval that is an random value between -30% and 30% of check_interval
        let check_interval =
            CHECK_INTERVAL + (CHECK_INTERVAL * (rand::random::<f64>() - 0.5) * 0.6).round();
        // Like update_interval
        let update_interval =
            UPDATE_INTERVAL + (UPDATE_INTERVAL * (rand::random::<f64>() - 0.5) * 0.6).round();

        debug!(
            "Locking pool {} with {:?} lock, check_interval: {}, update_interval: {}",
            self.name, lock_type, check_interval, update_interval
        );

        // We start by waiting for the lock to be free or compatible
        wait_for_lock(
            &self.lock_file,
            check_interval as u64,
            MAX_WAIT_TIME,
            UPDATE_INTERVAL as u64,
            &self.name,
            &self.uuid.to_string(),
            lock_type,
        )
        .await?;

        debug!("{:?} lock acquired for pool {}", lock_type, self.name);

        self.locked = true;

        // Update the lock file every n seconds
        let abort_handle = update_lock_file_thread(
            &self.lock_file,
            update_interval as u64,
            &self.uuid.to_string(),
        );
        self.abort_handle.replace(abort_handle);

        Ok(self)
    }
}

impl Drop for PoolLock {
    fn drop(&mut self) {
        debug!("Dropping lock for pool {}", self.name);

        if let Some(abort_handle) = self.abort_handle.take() {
            debug!("Stop update lock file thread of {}", self.name);
            abort_handle.abort();
        }

        // If locked, we need to remove our lock entry from the lock file
        if self.locked {
            // Use sync version - simple and deterministic
            if let Err(e) = remove_lock_entry_sync(&self.lock_file, &self.uuid.to_string()) {
                error!("Failed to remove lock entry for {}: {}", self.name, e);
            } else {
                debug!(
                    "Removed lock entry for {} from {}",
                    self.name,
                    self.lock_file.display()
                );
            }
        }
    }
}

fn is_stale_lock<P: AsRef<Path>>(
    lock: LockEntry,
    update_interval: u64,
    lock_file: P,
    current_lock_name: &str,
) -> bool {
    let timestamp_check = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time before UNIX EPOCH")
        .as_secs()
        - 3 * update_interval;

    if lock.timestamp < timestamp_check {
        error!(
            "Stale lock found in {} from pid {} ({}), will be removed for {current_lock_name}",
            lock_file.as_ref().display(),
            lock.pid,
            lock.lock_name,
        );
        true
    } else {
        debug!(
            "Active lock in file: pid={}, name={}, type={:?}, {current_lock_name} waiting",
            lock.pid,
            lock.lock_name,
            if lock.lock_type == 0 {
                "Shared"
            } else {
                "Exclusive"
            }
        );
        false
    }
}

fn filter_stale_lock<P: AsRef<Path>>(
    lock_file_data: LockFileData,
    update_interval: u64,
    lock_file: P,
    current_lock_name: &str,
) -> LockFileData {
    let filtered_locks: Vec<LockEntry> = lock_file_data
        .locks
        .into_iter()
        .filter(|lock| !is_stale_lock(lock.clone(), update_interval, &lock_file, current_lock_name))
        .collect();

    LockFileData {
        locks: filtered_locks,
    }
}

/// Reads the lock file and parses its content.
///
/// # Arguments
/// * `lock_file` - The path to the lock file to read.
///
/// # Returns
///
/// * `Ok((File, LockFileData))` if the lock file is successfully read and parsed.
/// * `Err(eyre::Report)` if an error occurs during reading or parsing.
///
/// # Errors
///
/// This function returns an error if:
/// * The lock file cannot be opened (permission)
/// * The lock file cannot be read (IO error)
/// * The lock file content cannot be parsed (protobuf decode error)
async fn read_file_lock(lock_file: &Path) -> Result<(File, LockFileData)> {
    let mut file_lock = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(lock_file)
        .await?;
    // Lock until the file can be read
    debug!("System lock file {} for read", lock_file.display());
    file_lock.lock_exclusive()?;

    let size = file_lock.metadata().await.map(|m| m.len()).unwrap_or(0);
    let mut buf = Vec::new();
    buf.try_reserve_exact(size as usize)?;
    file_lock.read_to_end(&mut buf).await?;

    if buf.is_empty() {
        return Ok((file_lock, LockFileData { locks: Vec::new() }));
    }

    let lock_file_data = LockFileData::decode(&buf[..])?;
    Ok((file_lock, lock_file_data))
}

async fn write_file_lock(mut lock_file: File, lock_file_data: LockFileData) -> Result<()> {
    // Encode the updated lock file data
    let mut buf = Vec::new();
    lock_file_data.encode(&mut buf)?;
    // Write the updated data back to the file atomically
    lock_file.seek(SeekFrom::Start(0)).await?;
    lock_file.set_len(buf.len() as u64).await?;
    lock_file.write_all(&buf).await?;
    lock_file.flush().await?;

    Ok(())
}

/// Removes a specific lock entry from the lock file.
///
/// # Arguments
/// * `lock_file` - The path to the lock file.
/// * `lock_name` - The name of the lock to remove.
/// * `pid` - The process ID of the lock to remove.
///
/// # Returns
///
/// * `Ok(())` if the lock entry is successfully removed.
/// Removes a specific lock entry from the lock file (synchronous version).
///
/// # Arguments
/// * `lock_file` - The path to the lock file.
/// * `lock_uuid` - The unique identifier of the lock to remove.
///
/// # Returns
///
/// * `Ok(())` if the lock entry is successfully removed.
/// * `Err(eyre::Report)` if an error occurs during reading or writing.
///
/// # Errors
///
/// Returns an error if the lock file cannot be read or written.
fn remove_lock_entry_sync(lock_file: &Path, lock_uuid: &str) -> Result<()> {
    debug!(
        "Removing lock entry with UUID {} from {}",
        lock_uuid,
        lock_file.display()
    );

    let mut file_lock = std::fs::File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(lock_file)?;
    file_lock.lock_exclusive()?;

    let size = file_lock.metadata().map(|m| m.len()).unwrap_or(0);
    let mut buf = Vec::new();
    buf.try_reserve_exact(size as usize)?;
    file_lock.read_to_end(&mut buf)?;

    let lock_file_data = LockFileData::decode(&buf[..])?;

    // Filter out our specific lock entry using UUID
    let filtered_locks: Vec<LockEntry> = lock_file_data
        .locks
        .into_iter()
        .filter(|lock| lock.lock_uuid != lock_uuid)
        .collect();

    // If no locks remain, remove the file entirely
    if filtered_locks.is_empty() {
        let _ = std::fs::remove_file(lock_file);
        debug!("Removed empty lock file {}", lock_file.display());
        return Ok(());
    }

    // Otherwise, update the file with remaining locks
    let updated_lock_data = LockFileData {
        locks: filtered_locks,
    };

    // Write atomically using sync version
    let mut buf = Vec::new();
    updated_lock_data.encode(&mut buf)?;

    file_lock.seek(SeekFrom::Start(0))?;
    file_lock.set_len(buf.len() as u64)?;
    file_lock.write_all(&buf)?;
    file_lock.flush()?;

    debug!(
        "Updated lock file {} after removing entry with UUID {}",
        lock_file.display(),
        lock_uuid
    );

    Ok(())
}

/// Updates only the timestamp of a specific lock entry in the lock file.
///
/// # Arguments
/// * `lock_file` - The path to the lock file to update.
/// * `lock_uuid` - The unique identifier of the lock to update.
///
/// # Returns
///
/// * `Ok(())` if the lock file is successfully updated.
/// * `Err(eyre::Report)` if an error occurs during reading, encoding, or writing.
///
/// # Errors
///
/// Returns an error if the lock file cannot be read, the lock cannot be found, or the file cannot be written.
async fn update_specific_lock_timestamp(lock_file: &Path, lock_uuid: &str) -> Result<()> {
    let (file, mut lock_file_data) = read_file_lock(lock_file).await?;

    let current_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    // Update timestamp only for our specific lock using UUID
    let mut found = false;
    for lock in &mut lock_file_data.locks {
        if lock.lock_uuid == lock_uuid {
            lock.timestamp = current_timestamp;
            found = true;
            break;
        }
    }

    if !found {
        return Err(eyre::eyre!(
            "Lock entry with UUID {} not found in lock file",
            lock_uuid
        ));
    }

    // Encode the updated lock file data
    write_file_lock(file, lock_file_data).await?;
    debug!(
        "Updated timestamp for lock with UUID {} in {}",
        lock_uuid,
        lock_file.display()
    );

    Ok(())
}

/// Checks if a new lock is compatible with existing locks.
///
/// # Rules
/// - Shared locks are compatible with other shared locks
/// - Exclusive locks are not compatible with any other lock
/// - Any lock is not compatible with an exclusive lock
///
/// # Arguments
/// * `existing_locks` - The existing locks in the lock file
/// * `new_lock_type` - The type of the new lock to check
///
/// # Returns
/// `true` if the new lock is compatible with all existing locks, `false` otherwise
fn is_lock_compatible(existing_locks: &[LockEntry], new_lock_type: LockType) -> bool {
    // If there are no existing locks, any new lock is compatible
    if existing_locks.is_empty() {
        return true;
    }

    // If the new lock is exclusive, it's only compatible if there are no existing locks
    if new_lock_type == LockType::Exclusive {
        return false;
    }

    // If the new lock is shared, it's compatible only if there are no exclusive locks
    for lock in existing_locks {
        if lock.lock_type == LockType::Exclusive as i32 {
            return false;
        }
    }

    // Shared lock is compatible with other shared locks
    true
}

/// Spawn a thread that updates only this specific lock's timestamp every n seconds.
fn update_lock_file_thread(
    lock_file: &Path,
    update_interval: u64,
    lock_uuid: &str,
) -> tokio::task::AbortHandle {
    let lock_file = lock_file.to_path_buf();
    let lock_uuid = lock_uuid.to_string();

    let handle = tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(update_interval)).await;
            debug!("Update lock file timestamp for UUID {}", lock_uuid);
            let result = update_specific_lock_timestamp(&lock_file, &lock_uuid).await;
            if let Err(e) = result {
                error!(
                    "Failed to update lock file timestamp for UUID {}: {}",
                    lock_uuid, e
                );
                // If we can't update our timestamp, the lock might be gone
                // Log this but don't panic - the process cleanup will handle it
            }
        }
    });

    handle.abort_handle()
}

/// Waits until a lock file becomes available, periodically checking and optionally removing stale locks.
///
/// # Arguments
/// * `lock_file` - The path to the lock file to monitor.
/// * `check_interval` - The interval in seconds between lock checks.
/// * `max_wait_time` - The maximum time in seconds to wait for the lock.
/// * `update_interval` - The expected update interval for the lock file (used to detect staleness).
/// * `name` - The name of the process or entity requesting the lock (for logging).
/// * `lock_uuid` - The unique identifier for this lock instance.
/// * `lock_type` - The type of lock being requested (shared or exclusive).
///
/// # Returns
///
/// * `Ok(())` if the lock is acquired.
/// * `Err(eyre::Report)` if the lock cannot be acquired within the maximum wait time or on error.
///
/// # Errors
///
/// Returns an error if the lock cannot be acquired, the lock file cannot be read, or if file operations fail.
async fn wait_for_lock(
    lock_file: &Path,
    check_interval: u64,
    max_wait_time: u64,
    update_interval: u64,
    name: &str,
    lock_uuid: &str,
    lock_type: LockType,
) -> Result<()> {
    let start = std::time::SystemTime::now();

    loop {
        {
            let (file, lock_file_data) = read_file_lock(lock_file).await?;

            // Filter out stale locks
            let mut active_locks =
                filter_stale_lock(lock_file_data, update_interval, lock_file, name);

            // Check if our lock is compatible with active locks
            if is_lock_compatible(&active_locks.locks, lock_type) {
                // Create our lock entry
                let new_entry = LockEntry {
                    pid: u64::from(std::process::id()),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_secs(),
                    lock_name: name.to_string(),
                    lock_type: lock_type as i32,
                    lock_uuid: lock_uuid.to_string(),
                };

                // Add our lock to the active locks
                active_locks.locks.push(new_entry);

                write_file_lock(file, active_locks).await?;
                break;
            }
        }

        // Check how many times we waited
        let elapsed = start.elapsed().unwrap().as_secs();
        if elapsed > max_wait_time {
            return Err(eyre::eyre!(
                "Can't acquire lock after waiting for {} seconds",
                max_wait_time
            ));
        }

        warn!(
            "Lock is busy, waiting for {check_interval} seconds for {name} with lock type {:?}",
            lock_type
        );
        tokio::time::sleep(std::time::Duration::from_secs(check_interval)).await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use test_log::test;

    // Note: Some tests in this module may take significant time (up to 90 seconds)
    // due to realistic timeout values used in production. This is expected behavior
    // and ensures tests validate real-world lock contention scenarios.

    #[test]
    fn test_pool_lock_new() {
        let path = Path::new("./data/");
        let lock = PoolLock::new_with_name(&path, "test_pool_lock_new");

        assert_eq!(lock.lock_file, path.join("lock"));
        assert!(!lock.locked);
        assert!(lock.abort_handle.is_none());
    }

    #[test(tokio::test)]
    async fn test_pool_lock_lock() {
        let path = Path::new("./data/");
        let lock = PoolLock::new_with_name(&path, "test_pool_lock_lock");
        let result = lock.lock().await;

        assert!(result.is_ok());

        let locked_pool = result.unwrap();
        assert!(locked_pool.locked);
        assert!(locked_pool.abort_handle.is_some());

        assert!(path.join("lock").exists());
    }

    #[test(tokio::test)]
    async fn test_pool_lock_drop() {
        let path = Path::new("./data/test_pool_lock_drop");
        // Ensure directory exists
        let _ = std::fs::create_dir_all(&path);

        let lock = PoolLock::new_with_name(&path, "test_pool_lock_drop");
        let result = lock.lock().await;

        assert!(result.is_ok());

        let locked_pool = result.unwrap();

        assert!(locked_pool.locked);
        assert!(locked_pool.abort_handle.is_some());

        assert!(path.join("lock").exists());
        // Dropping the lock should remove our specific lock entry immediately (sync)
        drop(locked_pool);
        // No need to sleep - sync drop is immediate!
        assert!(!path.join("lock").exists());
    }

    #[test(tokio::test)]
    async fn test_pool_lock_blocked() {
        // This test may take up to 90 seconds to complete due to stale lock detection
        // (3 * UPDATE_INTERVAL = 3 * 30 = 90 seconds)
        // This is expected behavior and matches production timeouts

        // Create a first lock
        let path = Path::new("./data/test_pool_lock_blocked");

        // Create directory if it doesn't exist
        std::fs::create_dir_all(path).ok();

        let lock = PoolLock::new_with_name(&path, "test_pool_lock_blocked_1");

        let result = lock.lock().await;

        assert!(result.is_ok());

        // Now we try to create a second lock on the same file,
        // wait 40 seconds to be sure that the first lock is still active
        // and drop the first lock, the second lock should release 5 secondes after
        let path2 = path.to_path_buf();
        let handle = tokio::spawn(async move {
            let lock2 = PoolLock::new_with_name(&path2, "test_pool_lock_blocked_2");
            let result = lock2.lock().await;
            assert!(result.is_ok());

            // Now we have the lock
            result.unwrap()
        });

        tokio::time::sleep(std::time::Duration::from_secs(40)).await;

        drop(result.unwrap());

        let lock2 = handle.await.unwrap();

        // Check we have the lock
        assert!(path.join("lock").exists());

        drop(lock2);

        // Since this was the last lock, the file should be removed
        // With synchronous drop, this should be immediate
        assert!(!path.join("lock").exists());
    }

    #[test(tokio::test)]
    async fn test_pool_lock_shared() {
        let path = Path::new("./data");
        let lock = PoolLock::new_with_name(&path, "test_pool_lock_shared");
        let result = lock.lock_shared().await;

        assert!(result.is_ok());

        let locked_pool = result.unwrap();
        assert!(locked_pool.locked);
        assert!(locked_pool.abort_handle.is_some());

        assert!(path.join("lock").exists());
    }

    #[test(tokio::test)]
    async fn test_pool_lock_exclusive() {
        let path = Path::new("./data");
        let lock = PoolLock::new_with_name(&path, "test_pool_lock_exclusive");
        let result = lock.lock_exclusive().await;

        assert!(result.is_ok());

        let locked_pool = result.unwrap();
        assert!(locked_pool.locked);
        assert!(locked_pool.abort_handle.is_some());

        assert!(path.join("lock").exists());
    }

    #[test(tokio::test)]
    async fn test_pool_lock_shared_compatibility() {
        // Create a first shared lock
        let path = Path::new("./data");
        let lock1 = PoolLock::new_with_name(&path, "test_shared_locks_1");
        let result1 = lock1.lock_shared().await;
        assert!(result1.is_ok());
        let locked_pool1 = result1.unwrap();

        // Try to create a second shared lock - should succeed
        let lock2 = PoolLock::new_with_name(&path, "test_shared_locks_2");
        let result2 = lock2.lock_shared().await;
        assert!(result2.is_ok());
        let locked_pool2 = result2.unwrap();

        // Both locks should be active
        assert!(locked_pool1.locked);
        assert!(locked_pool2.locked);

        // The lock file should exist and contain both locks
        assert!(path.join("lock").exists()); // Cleanup - drop one lock first
        drop(locked_pool2);
        // No sleep needed - sync drop is immediate!

        // File should still exist with the remaining lock
        assert!(path.join("lock").exists());
    }

    #[test(tokio::test)]
    async fn test_pool_lock_exclusive_incompatibility() {
        // Create an exclusive lock
        let path = Path::new("./data/test_pool_lock_exclusive_incompatibility");
        // Create directory if it doesn't exist
        std::fs::create_dir_all(path).ok();

        let lock = PoolLock::new_with_name(&path, "test_pool_lock_blocked_1");

        let result = lock.lock_exclusive().await;

        assert!(result.is_ok());

        // Now we try to create a second lock on the same file,
        // wait 40 seconds to be sure that the first lock is still active
        // and drop the first lock, the second lock should release 5 secondes after
        let path2 = path.to_path_buf();
        let handle = tokio::spawn(async move {
            let lock2 = PoolLock::new_with_name(&path2, "test_pool_lock_blocked_2");
            let result = lock2.lock_exclusive().await;
            assert!(result.is_ok());

            // Now we have the lock
            result.unwrap()
        });

        tokio::time::sleep(std::time::Duration::from_secs(40)).await;

        drop(result.unwrap());

        let lock2 = handle.await.unwrap();

        // Check we have the lock
        assert!(path.join("lock").exists());

        drop(lock2);

        // Since this was the last lock, the file should be removed
        // With synchronous drop, this should be immediate
        assert!(!path.join("lock").exists());
    }

    #[test(tokio::test)]
    async fn test_pool_lock_shared_exclusive_incompatibility() {
        // Create an exclusive lock
        let path = Path::new("./data/test_pool_lock_exclusive_incompatibility");
        // Create directory if it doesn't exist
        std::fs::create_dir_all(path).ok();

        let lock = PoolLock::new_with_name(&path, "test_pool_lock_blocked_1");

        let result = lock.lock_shared().await;

        assert!(result.is_ok());

        // Now we try to create a second lock on the same file,
        // wait 40 seconds to be sure that the first lock is still active
        // and drop the first lock, the second lock should release 5 secondes after
        let path2 = path.to_path_buf();
        let handle = tokio::spawn(async move {
            let lock2 = PoolLock::new_with_name(&path2, "test_pool_lock_blocked_2");
            let result = lock2.lock_exclusive().await;
            assert!(result.is_ok());

            // Now we have the lock
            result.unwrap()
        });

        tokio::time::sleep(std::time::Duration::from_secs(40)).await;

        drop(result.unwrap());

        let lock2 = handle.await.unwrap();

        // Check we have the lock
        assert!(path.join("lock").exists());

        drop(lock2);

        // Since this was the last lock, the file should be removed
        // With synchronous drop, this should be immediate
        assert!(!path.join("lock").exists());
    }
}

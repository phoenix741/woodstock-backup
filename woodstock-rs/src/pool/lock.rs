use eyre::Result;
use log::{debug, error, warn};
use prost::Message;
use std::path::{Path, PathBuf};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

#[derive(Clone, PartialEq, ::prost::Message)]
struct LockFileData {
    #[prost(uint64, tag = "1")]
    pub pid: u64,
    #[prost(uint64, tag = "2")]
    pub timestamp: u64,
    #[prost(string, tag = "3")]
    pub lock_name: String,
}

const CHECK_INTERVAL: u64 = 5; // seconds
const UPDATE_INTERVAL: u64 = 30; // seconds
const MAX_WAIT_TIME: u64 = 3_600; // seconds

/// The goal is to propose method that can be used to lock a resource.
///
/// The resource that can be locked is the pool. The pool can be locked for the following reasons:
/// - when the refcnt is updated (write lock)
/// - when unused file are removed (write lock)
/// - when a file is read (read lock)
/// - when the refcnt is read (read lock)
/// - when the refcnt is checked (read lock)
/// - when a file is added (added lock)
///
/// When a file is added, the refcnt isn't modified, a read lock should be enough.
///
/// The `async_fd_lock` crate will be used when file storage is used as the pool.
pub struct PoolLock {
    name: String,
    lock_file: PathBuf,
    locked: bool,
    abort_handle: Option<tokio::task::AbortHandle>,
}

impl PoolLock {
    pub fn new_with_name(path: &Path, name: &str) -> Self {
        PoolLock {
            name: name.to_string(),
            lock_file: path.join("lock"),
            locked: false,
            abort_handle: None,
        }
    }

    pub fn new(path: &Path) -> Self {
        PoolLock {
            name: path.to_str().map(|s| s.to_string()).unwrap_or_default(),
            lock_file: path.join("lock"),
            locked: false,
            abort_handle: None,
        }
    }

    pub async fn lock(mut self) -> Result<Self> {
        // Add an epsilon value to check_interval that is an random value between -30% and 30% of check_interval
        let check_interval = CHECK_INTERVAL
            + (CHECK_INTERVAL as f64 * (rand::random::<f64>() - 0.5) * 0.6).round() as u64;
        // Like update_interval
        let update_interval = UPDATE_INTERVAL
            + (UPDATE_INTERVAL as f64 * (rand::random::<f64>() - 0.5) * 0.6).round() as u64;

        debug!(
            "Locking pool {}, with check_interval: {}, update_interval: {}",
            self.name, check_interval, update_interval
        );

        // We start by waiting for the lock to be free
        wait_for_lock(
            &self.lock_file,
            check_interval,
            MAX_WAIT_TIME,
            UPDATE_INTERVAL,
            &self.name,
        )
        .await?;

        debug!("Lock acquired for pool {}", self.name);

        self.locked = true;

        // Update the lock file every n seconds
        let abort_handle =
            update_lock_file_thread(&self.lock_file, update_interval, &self.name).await;
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

        // If lock file, we can remove it
        if self.locked {
            // Remove the lock file
            debug!(
                "Remove lock file {} for {}",
                self.lock_file.display(),
                self.name
            );
            let _ = std::fs::remove_file(&self.lock_file);
        }
    }
}

async fn create_lock_file(lock_file: &Path, name: &str) -> Result<bool> {
    debug!("Try to create lock file {} for {name}", lock_file.display());

    let lock_file_data = LockFileData {
        pid: std::process::id() as u64,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        lock_name: name.to_string(),
    };

    // Convert to protobuf
    let mut buf = Vec::new();
    lock_file_data.encode(&mut buf)?;

    let mut options = OpenOptions::new();
    options.write(true).create_new(true);

    let mut file = match options.open(&lock_file).await {
        Ok(file) => file,
        Err(e) => {
            warn!("Can't create lock file: {e} for {name}");
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                return Ok(false);
            } else {
                return Err(e.into());
            }
        }
    };

    file.write_all(&buf).await?;
    Ok(true)
}

async fn read_file_lock(lock_file: &Path) -> Result<LockFileData> {
    let buf = tokio::fs::read(&lock_file).await?;
    let lock_file_data = LockFileData::decode(&buf[..])?;
    Ok(lock_file_data)
}

async fn update_lock_file(lock_file: &Path) -> Result<()> {
    let mut lock_file_data = read_file_lock(lock_file).await?;

    lock_file_data.timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    // Convert to protobuf
    let mut buf = Vec::new();
    lock_file_data.encode(&mut buf)?;

    tokio::fs::write(&lock_file, buf).await?;
    Ok(())
}

/// Spawn a thread that update the lock file every n seconds.
async fn update_lock_file_thread(
    lock_file: &Path,
    update_interval: u64,
    name: &str,
) -> tokio::task::AbortHandle {
    let lock_file = lock_file.to_path_buf();
    let name = name.to_string();

    let handle = tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(update_interval)).await;
            debug!("Update lock file for {}", name);
            update_lock_file(&lock_file).await.unwrap();
        }
    });

    handle.abort_handle()
}

async fn wait_for_lock(
    lock_file: &Path,
    check_interval: u64,
    max_wait_time: u64,
    update_interval: u64,
    name: &str,
) -> Result<()> {
    let start = std::time::SystemTime::now();

    loop {
        let lock_file_data = read_file_lock(lock_file).await;

        // If no file, lock is free
        if lock_file_data.is_err() {
            let created = create_lock_file(lock_file, name).await?;
            if created {
                break;
            }
        }

        if let Ok(lock_file_data) = lock_file_data {
            debug!(
                "Lock file is owned by pid: {} at {} by {name}",
                lock_file_data.pid, lock_file_data.timestamp
            );

            // If the timestamp is too old (not refreshed in the last 3 * update_interval), we can remove the lock
            let timestamp_check = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs()
                - 3 * update_interval;
            if lock_file_data.timestamp < timestamp_check {
                error!(
                    "Stale lock file {} from {}, removing it for {name}",
                    lock_file.display(),
                    lock_file_data.lock_name,
                );
                // Remove old lock file
                let _ = tokio::fs::remove_file(lock_file).await;

                // Create the lock file
                let created = create_lock_file(lock_file, name).await?;
                if created {
                    break;
                }
            }
        }

        // Check how many times we waited
        let elapsed = start.elapsed().unwrap().as_secs();
        if elapsed > max_wait_time {
            return Err(eyre::eyre!("Can't acquire lock"));
        }

        warn!("Lock is busy, waiting for {check_interval} seconds {name}");
        tokio::time::sleep(std::time::Duration::from_secs(check_interval)).await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use test_log::test;

    #[test]
    fn test_pool_lock_new() {
        let path = Path::new("./data/");
        let lock = PoolLock::new_with_name(path, "test_pool_lock_new");

        assert_eq!(lock.lock_file, path.join("lock"));
        assert!(!lock.locked);
        assert!(lock.abort_handle.is_none());
    }

    #[test(tokio::test)]
    async fn test_pool_lock_lock() {
        let path = Path::new("./data/");
        let lock = PoolLock::new_with_name(path, "test_pool_lock_lock");
        let result = lock.lock().await;

        assert!(result.is_ok());

        let locked_pool = result.unwrap();
        assert!(locked_pool.locked);
        assert!(locked_pool.abort_handle.is_some());

        assert!(path.join("lock").exists());
    }

    #[test(tokio::test)]
    async fn test_pool_lock_drop() {
        let path = Path::new("./data/");
        let lock = PoolLock::new_with_name(path, "test_pool_lock_drop");
        let result = lock.lock().await;

        assert!(result.is_ok());

        let locked_pool = result.unwrap();

        assert!(locked_pool.locked);
        assert!(locked_pool.abort_handle.is_some());

        assert!(path.join("lock").exists());
        // Dropping the lock should remove the lock file
        drop(locked_pool);
        assert!(!path.join("lock").exists());
    }

    #[test(tokio::test)]
    async fn test_pool_lock_blocked() {
        // Create a first lock
        let path = Path::new("./data/");
        let lock = PoolLock::new_with_name(path, "test_pool_lock_blocked_1");

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

        assert!(!path.join("lock").exists());
    }
}
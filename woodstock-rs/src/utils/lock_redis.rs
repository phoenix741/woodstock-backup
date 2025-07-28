use eyre::{Result, WrapErr};
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client, Script};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, warn};
use uuid::Uuid;

use crate::utils::path::mangle_path;

/// Type of lock for the pool
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LockType {
    /// Shared lock, allows concurrent compatible operations (backup, deletion, compaction)
    Shared = 0,
    /// Exclusive lock, prevents any other operation (cleaning, integrity verification)
    Exclusive = 1,
}

/// Interval in seconds between lock heartbeat updates
const HEARTBEAT_INTERVAL: u64 = 10;
/// TTL in seconds for lock keys in Redis.
/// Exposed publicly so callers can derive a sensible wait timeout: waiting LOCK_TTL seconds
/// guarantees that any lock whose heartbeat has stopped will have expired.
pub const LOCK_TTL: u64 = 30;
/// Interval in seconds between lock availability checks
const CHECK_INTERVAL: u64 = 5;
/// Maximum time in seconds to wait for a lock to be released
const MAX_WAIT_TIME: u64 = 3_600;
/// Timeout in seconds for lock cleanup in Drop
const DROP_CLEANUP_TIMEOUT: u64 = 5;

/// Lua script to atomically release an exclusive lock if owned by the caller
const RELEASE_EXCLUSIVE_SCRIPT: &str = r#"
    if redis.call("get", KEYS[1]) == ARGV[1] then
        return redis.call("del", KEYS[1])
    else
        return 0
    end
"#;

/// Lua script to atomically acquire a shared lock.
/// Checks for an existing exclusive lock, then atomically adds to the shared set
/// and creates the session key — preventing TOCTOU races.
/// KEYS: [1]=exclusive_key, [2]=shared_key, [3]=session_key
/// ARGV: [1]=uuid, [2]=ttl_seconds
const ACQUIRE_SHARED_SCRIPT: &str = r#"
    if redis.call("exists", KEYS[1]) == 1 then
        return 0
    end
    redis.call("sadd", KEYS[2], ARGV[1])
    redis.call("setex", KEYS[3], ARGV[2], "1")
    return 1
"#;

/// Lua script to atomically extend TTL of an exclusive lock if owned by the caller
const EXTEND_EXCLUSIVE_SCRIPT: &str = r#"
    if redis.call("get", KEYS[1]) == ARGV[1] then
        return redis.call("pexpire", KEYS[1], ARGV[2])
    else
        return 0
    end
"#;

/// Represents a Redis-based lock for a resource, such as a backup pool.
///
/// The lock ensures that only one process can access the resource exclusively,
/// or multiple processes can access it with shared locks.
///
/// This implementation mirrors the behavior of the filesystem-based `PoolLock`
/// but uses Redis for distributed locking across multiple processes.
pub struct PoolLockRedis {
    /// Redis client for connection management
    client: Client,
    /// Connection for async operations
    conn: Arc<Mutex<MultiplexedConnection>>,
    /// Resource identifier (e.g., pool path)
    resource: String,
    /// Operation name identifier (e.g., "fsck", "backup", "restore")
    operation_name: String,
    /// Process ID that owns this lock
    pid: u64,
    /// Unique identifier for this lock instance
    uuid: Uuid,
    /// Type of lock (shared or exclusive)
    lock_type: Option<LockType>,
    /// Flag indicating whether the lock is currently held
    locked: bool,
    /// Handle for the background task that updates the lock heartbeat
    abort_handle: Option<tokio::task::AbortHandle>,
    /// Flag to track if lock was lost
    lost_flag: Arc<Mutex<bool>>,
    /// Cancellation token triggered when the lock is lost (TTL expiry or heartbeat failure).
    /// Callers can use it with `tokio::select!` to abort work when the lock expires.
    cancellation_token: CancellationToken,
}

impl PoolLockRedis {
    /// Creates a new Redis lock instance.
    ///
    /// # Arguments
    ///
    /// * `redis_url` - The URL of the Redis server (e.g., "redis://localhost:6379")
    /// * `resource` - The resource identifier (e.g., pool path like "/data/pool1")
    /// * `operation_name` - The operation name (e.g., "fsck", "backup", "restore")
    ///
    /// # Returns
    ///
    /// Returns a result containing the new lock instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis connection cannot be established.
    pub async fn new(redis_url: &str, resource: &str, operation_name: &str) -> Result<Self> {
        let client = Client::open(redis_url)?;
        let conn = client.get_multiplexed_async_connection().await?;

        Ok(Self {
            client,
            conn: Arc::new(Mutex::new(conn)),
            resource: resource.to_string(),
            operation_name: operation_name.to_string(),
            pid: std::process::id() as u64,
            uuid: Uuid::new_v4(),
            lock_type: None,
            locked: false,
            abort_handle: None,
            lost_flag: Arc::new(Mutex::new(false)),
            cancellation_token: CancellationToken::new(),
        })
    }

    /// Creates a new Redis lock instance from a path.
    ///
    /// This is a convenience constructor that mangles the path to create a safe resource identifier.
    ///
    /// # Arguments
    ///
    /// * `redis_url` - The URL of the Redis server (e.g., "redis://localhost:6379")
    /// * `path` - The path to the resource (e.g., pool path like "/data/pool1")
    /// * `operation_name` - The operation name (e.g., "fsck", "backup", "restore")
    ///
    /// # Returns
    ///
    /// Returns a result containing the new lock instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis connection cannot be established.
    pub async fn new_with_path<P: AsRef<Path>>(
        redis_url: &str,
        path: P,
        operation_name: &str,
    ) -> Result<Self> {
        let resource = mangle_path(path);
        Self::new(redis_url, &resource, operation_name).await
    }

    /// Returns the Redis key for the exclusive lock.
    fn exclusive_key(&self) -> String {
        format!("lock:{}:exclusive", self.resource)
    }

    /// Returns the Redis key for the shared lock set.
    fn shared_key(&self) -> String {
        format!("lock:{}:shared", self.resource)
    }

    /// Returns the Redis key for this instance's session.
    fn session_key(&self) -> String {
        format!("lock:{}:session:{}", self.resource, self.uuid)
    }

    /// Returns the Redis key for a specific session UUID.
    fn session_key_for(&self, uuid: &str) -> String {
        format!("lock:{}:session:{}", self.resource, uuid)
    }

    /// Returns the Redis key for this instance's metadata.
    fn metadata_key(&self) -> String {
        format!("lock:{}:meta:{}", self.resource, self.uuid)
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

    /// Tries to acquire an exclusive lock on the pool without waiting.
    /// Returns None if the lock is already held by another process.
    ///
    /// # Returns
    ///
    /// Returns Some(Self) if the lock was acquired, None if it's already held.
    ///
    /// # Errors
    ///
    /// Returns an error if Redis operations fail.
    pub async fn try_lock_exclusive_nowait(mut self) -> Result<Option<Self>> {
        self.lock_type = Some(LockType::Exclusive);

        // Try to acquire the lock immediately without waiting
        if self.try_acquire_lock(LockType::Exclusive).await? {
            self.locked = true;

            // Store metadata for debugging
            self.store_metadata().await?;

            // Start the heartbeat to keep the lock alive
            let abort_handle = self.start_heartbeat();
            self.abort_handle = Some(abort_handle);

            Ok(Some(self))
        } else {
            Ok(None)
        }
    }

    /// Tries to acquire an exclusive lock, waiting up to `timeout` for it to become free.
    ///
    /// Unlike `try_lock_exclusive_nowait` (which returns `None` immediately if the lock is
    /// held) and `lock_exclusive` (which waits up to `MAX_WAIT_TIME = 1h`), this method
    /// waits at most `timeout`. Passing `Duration::from_secs(LOCK_TTL)` (30 s) is the
    /// recommended value: it guarantees that any lock whose heartbeat has stopped will have
    /// expired before we give up, without blocking indefinitely.
    ///
    /// Returns `Some(Self)` if the lock was acquired within the timeout, `None` otherwise.
    pub async fn try_lock_exclusive_wait(mut self, timeout: Duration) -> Result<Option<Self>> {
        self.lock_type = Some(LockType::Exclusive);
        let start = std::time::Instant::now();

        loop {
            if self.try_acquire_lock(LockType::Exclusive).await? {
                self.locked = true;
                self.store_metadata().await?;
                let abort_handle = self.start_heartbeat();
                self.abort_handle = Some(abort_handle);
                return Ok(Some(self));
            }

            let elapsed = start.elapsed();
            if elapsed >= timeout {
                return Ok(None);
            }

            // Sleep at most CHECK_INTERVAL but never past the deadline
            let remaining = timeout - elapsed;
            let wait = remaining.min(Duration::from_secs(CHECK_INTERVAL));
            warn!(
                "Lock {} is busy (operation: {}), retrying in {}s (timeout in {}s)",
                self.resource,
                self.operation_name,
                wait.as_secs(),
                remaining.as_secs(),
            );
            tokio::time::sleep(wait).await;
        }
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
        debug!(
            "Locking resource {} (operation: {}) with {:?} lock using Redis",
            self.resource, self.operation_name, lock_type
        );

        self.lock_type = Some(lock_type);

        // Wait for the lock to be available
        self.wait_for_lock(lock_type).await?;

        debug!(
            "{:?} lock acquired for resource {} (operation: {})",
            lock_type, self.resource, self.operation_name
        );

        self.locked = true;

        // Store metadata for debugging
        self.store_metadata().await?;

        // Start the heartbeat to keep the lock alive
        let abort_handle = self.start_heartbeat();
        self.abort_handle = Some(abort_handle);

        Ok(self)
    }

    /// Stores metadata about this lock instance in Redis for debugging purposes.
    ///
    /// Metadata includes:
    /// - operation_name: The name of the operation (e.g., "fsck", "backup")
    /// - pid: The process ID that owns the lock
    /// - timestamp: When the lock was acquired
    ///
    /// # Errors
    ///
    /// Returns an error if Redis operations fail.
    async fn store_metadata(&self) -> Result<()> {
        let mut conn = self.conn.lock().await;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        // Store metadata as a hash with TTL matching the lock
        let metadata_key = self.metadata_key();

        let _: () = redis::pipe()
            .atomic()
            .hset(&metadata_key, "operation", &self.operation_name)
            .ignore()
            .hset(&metadata_key, "pid", self.pid)
            .ignore()
            .hset(&metadata_key, "timestamp", timestamp)
            .ignore()
            .hset(
                &metadata_key,
                "lock_type",
                // SAFETY: lock_type is always Some when store_metadata is called — set in lock_internal before this
                format!("{:?}", self.lock_type.unwrap()),
            )
            .ignore()
            .expire(&metadata_key, LOCK_TTL as i64)
            .ignore()
            .query_async(&mut *conn)
            .await
            .wrap_err("Failed to store lock metadata")?;

        debug!(
            "Stored metadata for lock on {} (operation: {}, pid: {})",
            self.resource, self.operation_name, self.pid
        );

        Ok(())
    }

    /// Waits until the lock becomes available or compatible with existing locks.
    ///
    /// # Arguments
    ///
    /// * `lock_type` - The type of lock being requested
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the lock is acquired.
    /// * `Err(eyre::Report)` if the lock cannot be acquired within the maximum wait time.
    ///
    /// # Errors
    ///
    /// Returns an error if the lock cannot be acquired or if Redis operations fail.
    async fn wait_for_lock(&self, lock_type: LockType) -> Result<()> {
        let start = std::time::Instant::now();

        loop {
            // Try to acquire the lock
            if self.try_acquire_lock(lock_type).await? {
                return Ok(());
            }

            // Check timeout
            if start.elapsed().as_secs() > MAX_WAIT_TIME {
                return Err(eyre::eyre!(
                    "Cannot acquire {:?} lock for {} after waiting {} seconds",
                    lock_type,
                    self.resource,
                    MAX_WAIT_TIME
                ));
            }

            warn!(
                "Lock {} is busy, waiting {} seconds for {} with lock type {:?}",
                self.resource, CHECK_INTERVAL, self.resource, lock_type
            );

            tokio::time::sleep(Duration::from_secs(CHECK_INTERVAL)).await;
        }
    }

    /// Tries to acquire the lock based on compatibility rules.
    ///
    /// # Arguments
    ///
    /// * `lock_type` - The type of lock to acquire
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if the lock was acquired, `Ok(false)` if it's not available yet.
    ///
    /// # Errors
    ///
    /// Returns an error if Redis operations fail.
    async fn try_acquire_lock(&self, lock_type: LockType) -> Result<bool> {
        let mut conn = self.conn.lock().await;

        // Cleanup expired sessions from shared locks
        self.cleanup_expired_sessions(&mut conn, &self.shared_key())
            .await?;

        match lock_type {
            LockType::Exclusive => {
                // Check if there's already an exclusive lock
                let exclusive_exists: bool = conn
                    .exists(&self.exclusive_key())
                    .await
                    .wrap_err("Failed to check exclusive lock existence")?;
                if exclusive_exists {
                    return Ok(false);
                }

                // Check if there are any shared locks
                let shared_count: usize = conn
                    .scard(&self.shared_key())
                    .await
                    .wrap_err("Failed to check shared lock count")?;
                if shared_count > 0 {
                    return Ok(false);
                }

                // Acquire exclusive lock using SET NX PX (atomic)
                let acquired: bool = redis::cmd("SET")
                    .arg(&self.exclusive_key())
                    .arg(self.uuid.to_string())
                    .arg("NX")
                    .arg("PX")
                    .arg(LOCK_TTL * 1000)
                    .query_async::<bool>(&mut *conn)
                    .await
                    .wrap_err_with(|| {
                        format!("Failed to acquire exclusive lock for {}", self.resource)
                    })?;

                Ok(acquired)
            }
            LockType::Shared => {
                // Atomically check for exclusive lock, add to shared set and create
                // the session key in a single Lua script — prevents the TOCTOU race
                // where an exclusive lock could be acquired between our EXISTS check
                // and our SADD.
                let acquired: i32 = Script::new(ACQUIRE_SHARED_SCRIPT)
                    .key(&self.exclusive_key())
                    .key(&self.shared_key())
                    .key(&self.session_key())
                    .arg(self.uuid.to_string())
                    .arg(LOCK_TTL)
                    .invoke_async(&mut *conn)
                    .await
                    .wrap_err_with(|| {
                        format!(
                            "Failed to atomically acquire shared lock for {}",
                            self.resource
                        )
                    })?;

                Ok(acquired == 1)
            }
        }
    }

    /// Cleans up expired sessions from the shared lock set.
    ///
    /// # Arguments
    ///
    /// * `conn` - The Redis connection
    /// * `shared_key` - The key for the shared lock set
    ///
    /// # Errors
    ///
    /// Returns an error if Redis operations fail.
    async fn cleanup_expired_sessions(
        &self,
        conn: &mut MultiplexedConnection,
        shared_key: &str,
    ) -> Result<()> {
        let members: Vec<String> = conn
            .smembers(shared_key)
            .await
            .wrap_err("Failed to get shared lock members")?;

        for session_uuid in &members {
            let session_key = self.session_key_for(session_uuid);
            let exists: bool = conn
                .exists(&session_key)
                .await
                .wrap_err("Failed to check session key existence")?;

            if !exists {
                debug!(
                    "Removing expired session {} from shared lock {}",
                    session_uuid, self.resource
                );
                let _: () = conn
                    .srem(shared_key, session_uuid)
                    .await
                    .wrap_err("Failed to remove expired session")?;
            }
        }

        Ok(())
    }

    /// Starts the heartbeat task to keep the lock alive.
    ///
    /// # Returns
    ///
    /// Returns an abort handle for the heartbeat task.
    fn start_heartbeat(&self) -> tokio::task::AbortHandle {
        let client = self.client.clone();
        let resource = self.resource.clone();
        let uuid = self.uuid;
        // SAFETY: lock_type is always Some when start_heartbeat is called — set in lock_internal before this
        let lock_type = self.lock_type.expect("Lock type must be set");
        let lost_flag = self.lost_flag.clone();
        let metadata_key = self.metadata_key();
        let cancel_token = self.cancellation_token.clone();

        let handle = tokio::spawn(async move {
            let mut conn = match client.get_multiplexed_async_connection().await {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to get Redis connection for heartbeat: {}", e);
                    *lost_flag.lock().await = true;
                    return;
                }
            };

            let mut interval = tokio::time::interval(Duration::from_secs(HEARTBEAT_INTERVAL));

            // Use the shared constant for the script
            let extend_exclusive_script = Script::new(EXTEND_EXCLUSIVE_SCRIPT);

            loop {
                interval.tick().await;

                let result: Result<i32, _> = match lock_type {
                    LockType::Exclusive => {
                        let exclusive_key = format!("lock:{}:exclusive", resource);

                        extend_exclusive_script
                            .key(&exclusive_key)
                            .arg(uuid.to_string())
                            .arg(LOCK_TTL * 1000)
                            .invoke_async(&mut conn)
                            .await
                    }
                    LockType::Shared => {
                        let session_key = format!("lock:{}:session:{}", resource, uuid);

                        redis::cmd("EXPIRE")
                            .arg(&session_key)
                            .arg(LOCK_TTL)
                            .query_async::<bool>(&mut conn)
                            .await
                            .map(|success| if success { 1 } else { 0 })
                    }
                };

                // Also extend metadata TTL
                let _: Result<(), _> = redis::cmd("EXPIRE")
                    .arg(&metadata_key)
                    .arg(LOCK_TTL)
                    .query_async(&mut conn)
                    .await;

                match result {
                    Ok(1) => {
                        debug!("Heartbeat OK for lock {} (UUID: {})", resource, uuid);
                    }
                    Ok(0) => {
                        error!(
                            "Lock lost - key expired or was removed: {} (UUID: {})",
                            resource, uuid
                        );
                        *lost_flag.lock().await = true;
                        cancel_token.cancel();
                        break;
                    }
                    Err(e) => {
                        error!(
                            "Heartbeat failed for lock {} (UUID: {}): {}",
                            resource, uuid, e
                        );
                        *lost_flag.lock().await = true;
                        cancel_token.cancel();
                        break;
                    }
                    _ => {
                        error!(
                            "Unexpected heartbeat result for lock {} (UUID: {})",
                            resource, uuid
                        );
                        *lost_flag.lock().await = true;
                        cancel_token.cancel();
                        break;
                    }
                }
            }
        });

        handle.abort_handle()
    }

    /// Returns a reference to the cancellation token for this lock.
    ///
    /// The token is cancelled automatically when the lock is lost (TTL expiry or
    /// heartbeat failure). Use it with `tokio::select!` to abort long-running work
    /// as soon as the lock expires, rather than polling `check_valid`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let token = lock.cancellation_token().clone();
    /// tokio::select! {
    ///     biased;
    ///     res = do_work() => res,
    ///     _ = token.cancelled() => Err(eyre!("Lock lost")),
    /// }
    /// ```
    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation_token
    }

    /// Checks if the lock is still valid (hasn't been lost).
    ///
    /// This can be called periodically during long operations to ensure
    /// the lock is still held.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the lock is still valid, or an error if it was lost.
    ///
    /// # Errors
    ///
    /// Returns an error if the lock was lost due to heartbeat failure or expiration.
    pub async fn check_valid(&self) -> Result<()> {
        let lost = self.lost_flag.lock().await;
        if *lost {
            return Err(eyre::eyre!(
                "Lock was lost for resource: {} (UUID: {})",
                self.resource,
                self.uuid
            ));
        }
        Ok(())
    }

    /// Releases the lock explicitly.
    ///
    /// This is called automatically when the lock is dropped, but can be
    /// called explicitly if needed.
    ///
    /// # Errors
    ///
    /// Returns an error if the lock cannot be released due to Redis operations failing.
    pub async fn unlock(mut self) -> Result<()> {
        self.release().await
    }

    /// Internal release implementation used by both unlock() and Drop.
    async fn release(&mut self) -> Result<()> {
        if !self.locked {
            return Ok(());
        }

        debug!("Releasing lock for {} (UUID: {})", self.resource, self.uuid);

        // Stop the heartbeat
        if let Some(handle) = self.abort_handle.take() {
            debug!(
                "Stopping heartbeat for {} (UUID: {})",
                self.resource, self.uuid
            );
            handle.abort();
        }

        // SAFETY: lock_type is always Some for a locked instance — set in lock_internal before unlock()
        let lock_type = self.lock_type.expect("Lock type must be set");
        let mut conn = self.conn.lock().await;

        self.release_redis_keys(&mut conn, lock_type).await?;

        self.locked = false;

        debug!("Lock released for {} (UUID: {})", self.resource, self.uuid);

        Ok(())
    }

    /// Releases the lock keys in Redis based on lock type.
    async fn release_redis_keys(
        &self,
        conn: &mut MultiplexedConnection,
        lock_type: LockType,
    ) -> Result<()> {
        match lock_type {
            LockType::Exclusive => self.release_exclusive_lock(conn).await,
            LockType::Shared => self.release_shared_lock(conn).await,
        }
    }

    /// Releases an exclusive lock using Lua script for atomicity.
    async fn release_exclusive_lock(&self, conn: &mut MultiplexedConnection) -> Result<()> {
        // Use the shared constant for the script
        let release_script = Script::new(RELEASE_EXCLUSIVE_SCRIPT);

        let result: i32 = release_script
            .key(&self.exclusive_key())
            .arg(self.uuid.to_string())
            .invoke_async(conn)
            .await
            .wrap_err_with(|| format!("Failed to release exclusive lock for {}", self.resource))?;

        if result == 0 {
            warn!(
                "Exclusive lock for {} was not owned by this instance (UUID: {})",
                self.resource, self.uuid
            );
        }

        // Delete metadata key
        let _: () = conn
            .del(&self.metadata_key())
            .await
            .wrap_err_with(|| format!("Failed to delete metadata key for {}", self.resource))?;

        Ok(())
    }

    /// Releases a shared lock and cleans up the shared set if empty.
    async fn release_shared_lock(&self, conn: &mut MultiplexedConnection) -> Result<()> {
        // Remove from shared set
        let _: () = conn
            .srem(&self.shared_key(), self.uuid.to_string())
            .await
            .wrap_err_with(|| format!("Failed to remove shared lock for {}", self.resource))?;

        // Delete session key
        let _: () = conn
            .del(&self.session_key())
            .await
            .wrap_err_with(|| format!("Failed to delete session key for {}", self.resource))?;

        // Delete metadata key
        let _: () = conn
            .del(&self.metadata_key())
            .await
            .wrap_err_with(|| format!("Failed to delete metadata key for {}", self.resource))?;

        // Clean up the shared set if it's empty
        let count: usize = conn
            .scard(&self.shared_key())
            .await
            .wrap_err("Failed to check shared lock count")?;
        if count == 0 {
            let _: () = conn
                .del(&self.shared_key())
                .await
                .wrap_err("Failed to delete empty shared set")?;
        }

        Ok(())
    }

    /// Static helper for releasing exclusive lock (used in Drop).
    ///
    /// This method is static because it's called from Drop in a detached tokio task,
    /// where we can't borrow `self`. It replicates the logic from `release_exclusive_lock()`
    /// but works with raw parameters instead of self references.
    async fn release_exclusive_lock_static(
        conn: &mut MultiplexedConnection,
        resource: &str,
        uuid: Uuid,
    ) -> Result<()> {
        let exclusive_key = format!("lock:{}:exclusive", resource);
        let metadata_key = format!("lock:{}:meta:{}", resource, uuid);

        // Use the shared constant for the script
        let release_script = Script::new(RELEASE_EXCLUSIVE_SCRIPT);

        let result: i32 = release_script
            .key(&exclusive_key)
            .arg(uuid.to_string())
            .invoke_async(conn)
            .await
            .wrap_err_with(|| format!("Failed to release exclusive lock for {}", resource))?;

        if result == 0 {
            warn!(
                "Exclusive lock for {} was not owned by UUID {} during drop cleanup",
                resource, uuid
            );
        }

        // Delete metadata key
        let _: () = conn
            .del(&metadata_key)
            .await
            .wrap_err_with(|| format!("Failed to delete metadata key for {}", resource))?;

        Ok(())
    }

    /// Static helper for releasing shared lock (used in Drop).
    ///
    /// This method is static because it's called from Drop in a detached tokio task,
    /// where we can't borrow `self`. It replicates the logic from `release_shared_lock()`
    /// but works with raw parameters instead of self references.
    async fn release_shared_lock_static(
        conn: &mut MultiplexedConnection,
        resource: &str,
        uuid: Uuid,
    ) -> Result<()> {
        let shared_key = format!("lock:{}:shared", resource);
        let session_key = format!("lock:{}:session:{}", resource, uuid);
        let metadata_key = format!("lock:{}:meta:{}", resource, uuid);

        let _: () = conn
            .srem(&shared_key, uuid.to_string())
            .await
            .wrap_err_with(|| format!("Failed to remove shared lock for {}", resource))?;
        let _: () = conn
            .del(&session_key)
            .await
            .wrap_err_with(|| format!("Failed to delete session key for {}", resource))?;
        let _: () = conn
            .del(&metadata_key)
            .await
            .wrap_err_with(|| format!("Failed to delete metadata key for {}", resource))?;

        // Clean up the shared set if empty
        let count: usize = conn
            .scard(&shared_key)
            .await
            .wrap_err("Failed to check shared lock count")?;
        if count == 0 {
            let _: () = conn
                .del(&shared_key)
                .await
                .wrap_err("Failed to delete empty shared set")?;
        }

        Ok(())
    }
}

impl Drop for PoolLockRedis {
    fn drop(&mut self) {
        if !self.locked {
            return;
        }

        debug!(
            "Dropping Redis lock for {} (operation: {}, UUID: {})",
            self.resource, self.operation_name, self.uuid
        );

        // Stop the heartbeat immediately
        if let Some(handle) = self.abort_handle.take() {
            debug!(
                "Stopping heartbeat for {} (operation: {}, UUID: {})",
                self.resource, self.operation_name, self.uuid
            );
            handle.abort();
        }

        // Spawn a detached task to release the lock with timeout
        // We can't await in Drop, so we spawn a task that will complete independently
        let conn = self.conn.clone();
        let resource = self.resource.clone();
        let uuid = self.uuid;
        // SAFETY: lock_type is always Some in Drop — Drop is only reachable after lock_internal set it
        let lock_type = self.lock_type.expect("Lock type must be set");

        tokio::spawn(async move {
            // Add timeout to prevent indefinite blocking if Redis is down
            let cleanup = async {
                let mut conn_guard = conn.lock().await;

                match lock_type {
                    LockType::Exclusive => {
                        Self::release_exclusive_lock_static(&mut conn_guard, &resource, uuid).await
                    }
                    LockType::Shared => {
                        Self::release_shared_lock_static(&mut conn_guard, &resource, uuid).await
                    }
                }
            };

            match tokio::time::timeout(Duration::from_secs(DROP_CLEANUP_TIMEOUT), cleanup).await {
                Ok(Ok(_)) => {
                    debug!("Lock cleanup completed for {} (UUID: {})", resource, uuid);
                }
                Ok(Err(e)) => {
                    error!(
                        "Failed to release lock in drop for {} (UUID: {}): {}",
                        resource, uuid, e
                    );
                }
                Err(_) => {
                    error!(
                        "Lock cleanup timed out after {}s for {} (UUID: {})",
                        DROP_CLEANUP_TIMEOUT, resource, uuid
                    );
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use test_log::test;

    fn redis_url() -> String {
        let host = std::env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        format!("redis://{}:6379", host)
    }

    #[test(tokio::test)]
    async fn test_redis_lock_new() {
        let lock = PoolLockRedis::new(&redis_url(), "/test/test_new", "test_new").await;
        assert!(lock.is_ok());

        let lock = lock.unwrap();
        assert_eq!(lock.resource, "/test/test_new");
        assert_eq!(lock.operation_name, "test_new");
        assert!(!lock.locked);
        assert!(lock.abort_handle.is_none());
    }

    #[test(tokio::test)]
    async fn test_redis_lock_new_with_path() {
        let path = PathBuf::from("/test/path/to/resource");
        let lock = PoolLockRedis::new_with_path(&redis_url(), &path, "test_path").await;
        assert!(lock.is_ok());

        let lock = lock.unwrap();
        // The resource should be the mangled version of the path
        assert!(lock.resource.contains("test"));
        assert!(lock.resource.contains("path"));
        assert_eq!(lock.operation_name, "test_path");
        assert!(!lock.locked);
        assert!(lock.abort_handle.is_none());
    }

    #[test(tokio::test)]
    async fn test_redis_lock_exclusive() {
        let lock = PoolLockRedis::new(&redis_url(), "/test/test_exclusive", "test_exclusive")
            .await
            .unwrap();
        let result = lock.lock_exclusive().await;

        assert!(result.is_ok());

        let locked = result.unwrap();
        assert!(locked.locked);
        assert!(locked.abort_handle.is_some());
    }

    #[test(tokio::test)]
    async fn test_redis_lock_shared() {
        let lock = PoolLockRedis::new(&redis_url(), "/test/test_shared", "test_shared")
            .await
            .unwrap();
        let result = lock.lock_shared().await;

        assert!(result.is_ok());

        let locked = result.unwrap();
        assert!(locked.locked);
        assert!(locked.abort_handle.is_some());
    }

    #[test(tokio::test)]
    async fn test_redis_lock_shared_compatibility() {
        let lock1 = PoolLockRedis::new(
            &redis_url(),
            "/test/test_shared_compat",
            "test_shared_compat",
        )
        .await
        .unwrap();
        let locked1 = lock1.lock_shared().await.unwrap();

        // Second shared lock should succeed
        let lock2 = PoolLockRedis::new(
            &redis_url(),
            "/test/test_shared_compat",
            "test_shared_compat",
        )
        .await
        .unwrap();
        let locked2 = lock2.lock_shared().await.unwrap();

        assert!(locked1.locked);
        assert!(locked2.locked);

        // Cleanup
        drop(locked1);
        drop(locked2);
    }

    #[test(tokio::test)]
    async fn test_redis_lock_exclusive_blocks_shared() {
        let lock1 = PoolLockRedis::new(
            &redis_url(),
            "/test/test_exclusive_blocks",
            "test_exclusive_blocks",
        )
        .await
        .unwrap();
        let locked1 = lock1.lock_exclusive().await.unwrap();

        let lock2 = PoolLockRedis::new(
            &redis_url(),
            "/test/test_exclusive_blocks",
            "test_exclusive_blocks",
        )
        .await
        .unwrap();

        let handle = tokio::spawn(async move {
            let result = lock2.lock_shared().await;
            assert!(result.is_ok());
            result.unwrap()
        });

        // Wait a bit to ensure lock2 is waiting
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Release first lock
        drop(locked1);

        // Second lock should now succeed
        let locked2 = handle.await.unwrap();
        assert!(locked2.locked);
    }

    #[test(tokio::test)]
    async fn test_redis_lock_shared_blocks_exclusive() {
        let lock1 = PoolLockRedis::new(
            &redis_url(),
            "/test/test_shared_blocks",
            "test_shared_blocks",
        )
        .await
        .unwrap();
        let locked1 = lock1.lock_shared().await.unwrap();

        let lock2 = PoolLockRedis::new(
            &redis_url(),
            "/test/test_shared_blocks",
            "test_shared_blocks",
        )
        .await
        .unwrap();

        let handle = tokio::spawn(async move {
            let result = lock2.lock_exclusive().await;
            assert!(result.is_ok());
            result.unwrap()
        });

        // Wait to ensure lock2 is waiting
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Release first lock
        drop(locked1);

        // Second lock should now succeed
        let locked2 = handle.await.unwrap();
        assert!(locked2.locked);
    }

    #[test(tokio::test)]
    async fn test_redis_lock_heartbeat_maintains_lock() {
        let lock = PoolLockRedis::new(&redis_url(), "/test/test_heartbeat", "test_heartbeat")
            .await
            .unwrap();
        let locked = lock.lock_exclusive().await.unwrap();

        // Wait longer than the TTL to ensure heartbeat is working
        tokio::time::sleep(Duration::from_secs(LOCK_TTL + 5)).await;

        // Lock should still be valid
        assert!(locked.check_valid().await.is_ok());

        // Try to acquire another lock - should fail because first lock is still alive
        let lock2 = PoolLockRedis::new(&redis_url(), "/test/test_heartbeat", "test_heartbeat")
            .await
            .unwrap();

        let handle = tokio::spawn(async move { lock2.lock_exclusive().await });

        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should still be waiting
        assert!(!handle.is_finished());

        // Cleanup
        drop(locked);
        let _ = handle.await;
    }

    #[test(tokio::test)]
    async fn test_redis_lock_check_valid() {
        let lock = PoolLockRedis::new(&redis_url(), "/test/test_check_valid", "test_check_valid")
            .await
            .unwrap();
        let locked = lock.lock_exclusive().await.unwrap();

        // Initially should be valid
        assert!(locked.check_valid().await.is_ok());

        // After some time, should still be valid (heartbeat maintains it)
        tokio::time::sleep(Duration::from_secs(5)).await;
        assert!(locked.check_valid().await.is_ok());
    }

    #[test(tokio::test)]
    async fn test_try_lock_exclusive_nowait_success() {
        let lock = PoolLockRedis::new(&redis_url(), "/test/test_try_nowait_success", "test")
            .await
            .unwrap();
        let result = lock.try_lock_exclusive_nowait().await;

        assert!(result.is_ok());
        let locked = result.unwrap();
        assert!(locked.is_some());
        assert!(locked.unwrap().locked);
    }

    #[test(tokio::test)]
    async fn test_try_lock_exclusive_nowait_fails_when_locked() {
        let lock1 = PoolLockRedis::new(&redis_url(), "/test/test_try_nowait_fail", "test1")
            .await
            .unwrap();
        let _locked1 = lock1.lock_exclusive().await.unwrap();

        // Try to acquire lock without waiting - should fail
        let lock2 = PoolLockRedis::new(&redis_url(), "/test/test_try_nowait_fail", "test2")
            .await
            .unwrap();
        let result = lock2.try_lock_exclusive_nowait().await;

        assert!(result.is_ok());
        let locked2 = result.unwrap();
        assert!(locked2.is_none());
    }
}

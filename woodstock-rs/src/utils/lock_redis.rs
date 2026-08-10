use eyre::{Result, WrapErr};
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client, Script};
use std::fmt;
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LockOperation {
    Host(HostLockOperation),
    Pool(PoolLockOperation),
    Events,
    File(FileLockOperation),
    Import(ImportLockOperation),
    Internal(InternalLockOperation),
    Custom(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostLockOperation {
    Backup,
    Restore,
    Remove,
    Archive,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PoolLockOperation {
    SaveBackup,
    RemoveBackup,
    Fsck,
    ExecuteCleaning,
    ExecuteHashConversion,
    CompactRefcntManual,
    CheckCompression,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileLockOperation {
    Write,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImportLockOperation {
    Refcnt,
    Cleanup,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InternalLockOperation {
    InspectLockState,
}

impl HostLockOperation {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Backup => "backup",
            Self::Restore => "restore",
            Self::Remove => "remove",
            Self::Archive => "archive",
        }
    }
}

impl PoolLockOperation {
    fn as_str(&self) -> &'static str {
        match self {
            Self::SaveBackup => "save_backup",
            Self::RemoveBackup => "remove",
            Self::Fsck => "fsck",
            Self::ExecuteCleaning => "execute_cleaning",
            Self::ExecuteHashConversion => "execute_hash_conversion",
            Self::CompactRefcntManual => "compact_refcnt_manual",
            Self::CheckCompression => "check_compression",
        }
    }
}

impl FileLockOperation {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Write => "write",
        }
    }
}

impl ImportLockOperation {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Refcnt => "backuppc_importer_refcnt",
            Self::Cleanup => "backuppc_importer_cleanup",
        }
    }
}

impl InternalLockOperation {
    fn as_str(&self) -> &'static str {
        match self {
            Self::InspectLockState => "inspect_lock_state",
        }
    }
}

impl LockOperation {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Host(operation) => operation.as_str(),
            Self::Pool(operation) => operation.as_str(),
            Self::Events => "events",
            Self::File(operation) => operation.as_str(),
            Self::Import(operation) => operation.as_str(),
            Self::Internal(operation) => operation.as_str(),
            Self::Custom(value) => value.as_str(),
        }
    }
}

impl fmt::Display for LockOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for LockOperation {
    fn from(value: &str) -> Self {
        match value {
            "backup" => Self::Host(HostLockOperation::Backup),
            "restore" => Self::Host(HostLockOperation::Restore),
            "remove" => Self::Host(HostLockOperation::Remove),
            "archive" => Self::Host(HostLockOperation::Archive),
            "save_backup" => Self::Pool(PoolLockOperation::SaveBackup),
            "fsck" => Self::Pool(PoolLockOperation::Fsck),
            "execute_cleaning" => Self::Pool(PoolLockOperation::ExecuteCleaning),
            "execute_hash_conversion" => Self::Pool(PoolLockOperation::ExecuteHashConversion),
            "events" => Self::Events,
            "write" => Self::File(FileLockOperation::Write),
            "compact_refcnt_manual" => Self::Pool(PoolLockOperation::CompactRefcntManual),
            "check_compression" => Self::Pool(PoolLockOperation::CheckCompression),
            "backuppc_importer_refcnt" => Self::Import(ImportLockOperation::Refcnt),
            "backuppc_importer_cleanup" => Self::Import(ImportLockOperation::Cleanup),
            "inspect_lock_state" => Self::Internal(InternalLockOperation::InspectLockState),
            other => Self::Custom(other.to_string()),
        }
    }
}

impl From<String> for LockOperation {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveExclusiveLock {
    pub operation_name: Option<LockOperation>,
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

/// Lua script to atomically acquire an exclusive lock.
/// Refuses acquisition if an exclusive lock already exists or if any shared lock session remains.
/// KEYS: [1]=exclusive_key, [2]=shared_key
/// ARGV: [1]=uuid, [2]=ttl_millis
const ACQUIRE_EXCLUSIVE_SCRIPT: &str = r#"
    if redis.call("scard", KEYS[2]) > 0 then
        return 0
    end
    local result = redis.call("set", KEYS[1], ARGV[1], "NX", "PX", ARGV[2])
    if result then
        return 1
    end
    return 0
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
    /// Operation name identifier (e.g., fsck, backup, restore)
    operation_name: LockOperation,
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
    /// * `operation_name` - The operation name identifier
    ///
    /// # Returns
    ///
    /// Returns a result containing the new lock instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis connection cannot be established.
    pub async fn new(
        redis_url: &str,
        resource: &str,
        operation_name: impl Into<LockOperation>,
    ) -> Result<Self> {
        let client = Client::open(redis_url)?;
        let mut conn = client.get_multiplexed_async_connection().await?;
        let operation_name = operation_name.into();

        if tracing::enabled!(tracing::Level::DEBUG) {
            Self::log_connection_identity(&mut conn, "new", Some(resource)).await?;
        }

        Ok(Self {
            client,
            conn: Arc::new(Mutex::new(conn)),
            resource: resource.to_string(),
            operation_name,
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
    ///
    /// # Arguments
    ///
    /// * `redis_url` - The URL of the Redis server (e.g., "redis://localhost:6379")
    /// * `path` - The path to the resource (e.g., pool path like "/data/pool1")
    /// * `operation_name` - The operation name identifier
    ///
    ///
    /// Returns a result containing the new lock instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis connection cannot be established.
    pub async fn new_with_path<P: AsRef<Path>>(
        redis_url: &str,
        path: P,
        operation_name: impl Into<LockOperation>,
    ) -> Result<Self> {
        let resource = mangle_path(path);
        Self::new(redis_url, &resource, operation_name).await
    }

    async fn log_connection_identity(
        conn: &mut MultiplexedConnection,
        context: &str,
        resource: Option<&str>,
    ) -> Result<()> {
        let client_id: i64 = redis::cmd("CLIENT")
            .arg("ID")
            .query_async(conn)
            .await
            .wrap_err("Failed to inspect Redis client id")?;
        let info: String = redis::cmd("INFO")
            .arg("server")
            .query_async(conn)
            .await
            .wrap_err("Failed to inspect Redis server info")?;

        let run_id = info
            .lines()
            .find_map(|line| line.strip_prefix("run_id:"))
            .unwrap_or("unknown");
        let process_id = info
            .lines()
            .find_map(|line| line.strip_prefix("process_id:"))
            .unwrap_or("unknown");
        let tcp_port = info
            .lines()
            .find_map(|line| line.strip_prefix("tcp_port:"))
            .unwrap_or("unknown");

        debug!(
            "Redis connection identity [{}] resource={:?}: client_id={}, run_id={}, process_id={}, tcp_port={}",
            context, resource, client_id, run_id, process_id, tcp_port
        );

        Ok(())
    }

    /// Passively inspects whether a resource currently has any active Redis lock.
    ///
    /// Unlike `try_lock_exclusive_nowait`, this method never acquires or releases a
    /// lock itself. It only inspects the Redis keys already present for the resource.
    pub async fn has_active_lock(redis_url: &str, resource: &str) -> Result<bool> {
        let probe = Self::new(
            redis_url,
            resource,
            LockOperation::Internal(InternalLockOperation::InspectLockState),
        )
        .await?;
        probe.has_active_lock_internal().await
    }

    /// Passively inspects whether a resource currently has an active exclusive lock and,
    /// when available, returns the operation metadata associated with that lock.
    pub async fn active_exclusive_lock(
        redis_url: &str,
        resource: &str,
    ) -> Result<Option<ActiveExclusiveLock>> {
        let probe = Self::new(
            redis_url,
            resource,
            LockOperation::Internal(InternalLockOperation::InspectLockState),
        )
        .await?;
        probe.active_exclusive_lock_internal().await
    }

    /// Same as `active_exclusive_lock`, but accepts a filesystem path and applies the
    /// same mangling as `new_with_path`.
    pub async fn active_exclusive_lock_with_path<P: AsRef<Path>>(
        redis_url: &str,
        path: P,
    ) -> Result<Option<ActiveExclusiveLock>> {
        let resource = mangle_path(path);
        Self::active_exclusive_lock(redis_url, &resource).await
    }

    async fn has_active_lock_internal(&self) -> Result<bool> {
        let mut conn = self.conn.lock().await;

        self.cleanup_expired_sessions(&mut conn, &self.shared_key())
            .await?;

        let exclusive_exists: bool = conn
            .exists(self.exclusive_key())
            .await
            .wrap_err("Failed to inspect exclusive lock existence")?;
        if exclusive_exists {
            return Ok(true);
        }

        let shared_count: usize = conn
            .scard(self.shared_key())
            .await
            .wrap_err("Failed to inspect shared lock count")?;

        Ok(shared_count > 0)
    }

    async fn active_exclusive_lock_internal(&self) -> Result<Option<ActiveExclusiveLock>> {
        let mut conn = self.conn.lock().await;

        let owner: Option<String> = conn
            .get(self.exclusive_key())
            .await
            .wrap_err("Failed to inspect exclusive lock owner")?;

        let Some(owner) = owner else {
            return Ok(None);
        };

        let operation_name: Option<String> = conn
            .hget(self.metadata_key_for(&owner), "operation")
            .await
            .wrap_err("Failed to inspect exclusive lock metadata")?;

        Ok(Some(ActiveExclusiveLock {
            operation_name: operation_name.map(LockOperation::from),
        }))
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

    /// Returns the Redis key for metadata associated with a specific lock owner.
    fn metadata_key_for(&self, owner_uuid: &str) -> String {
        format!("lock:{}:meta:{}", self.resource, owner_uuid)
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
            self.finish_lock_acquisition().await?;
            Ok(Some(self))
        } else {
            Ok(None)
        }
    }

    /// Tries to acquire a shared lock, waiting up to `timeout` for it to become compatible.
    ///
    /// Unlike `lock_shared` (which waits up to `MAX_WAIT_TIME = 1h`), this method waits at
    /// most `timeout`. This is useful for backup operations that should tolerate a short-lived
    /// exclusive lock like cleanup, but should not block indefinitely behind a long-running fsck.
    ///
    /// Returns `Some(Self)` if the lock was acquired within the timeout, `None` otherwise.
    pub async fn try_lock_shared_wait(self, timeout: Duration) -> Result<Option<Self>> {
        self.try_lock_wait_internal(LockType::Shared, timeout, "Shared lock")
            .await
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
    pub async fn try_lock_exclusive_wait(self, timeout: Duration) -> Result<Option<Self>> {
        self.try_lock_wait_internal(LockType::Exclusive, timeout, "Exclusif Lock")
            .await
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

        self.finish_lock_acquisition().await?;

        Ok(self)
    }

    async fn try_lock_wait_internal(
        mut self,
        lock_type: LockType,
        timeout: Duration,
        busy_label: &str,
    ) -> Result<Option<Self>> {
        self.lock_type = Some(lock_type);
        let start = std::time::Instant::now();

        loop {
            if self.try_acquire_lock(lock_type).await? {
                self.finish_lock_acquisition().await?;
                return Ok(Some(self));
            }

            let elapsed = start.elapsed();
            if elapsed >= timeout {
                return Ok(None);
            }

            let remaining = timeout - elapsed;
            let wait = remaining.min(Duration::from_secs(CHECK_INTERVAL));
            debug!(
                "{} {} is busy (operation: {}), retrying in {}s (timeout in {}s)",
                busy_label,
                self.resource,
                self.operation_name,
                wait.as_secs(),
                remaining.as_secs(),
            );
            tokio::time::sleep(wait).await;
        }
    }

    async fn finish_lock_acquisition(&mut self) -> Result<()> {
        self.locked = true;
        self.store_metadata().await?;
        let abort_handle = self.start_heartbeat();
        self.abort_handle = Some(abort_handle);
        Ok(())
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
            .hset(&metadata_key, "operation", self.operation_name.as_str())
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

            debug!(
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
                self.log_lock_state(&mut conn, "Before exclusive acquire attempt")
                    .await?;

                let acquired: i32 = Script::new(ACQUIRE_EXCLUSIVE_SCRIPT)
                    .key(&self.exclusive_key())
                    .key(&self.shared_key())
                    .arg(self.uuid.to_string())
                    .arg(LOCK_TTL * 1000)
                    .invoke_async(&mut *conn)
                    .await
                    .wrap_err_with(|| {
                        format!(
                            "Failed to atomically acquire exclusive lock for {}",
                            self.resource
                        )
                    })?;

                self.log_lock_state(
                    &mut conn,
                    if acquired == 1 {
                        "After successful exclusive acquire"
                    } else {
                        "After failed exclusive acquire"
                    },
                )
                .await?;

                Ok(acquired == 1)
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
                warn!(
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

    async fn log_lock_state(&self, conn: &mut MultiplexedConnection, context: &str) -> Result<()> {
        let exclusive_key = self.exclusive_key();
        let shared_key = self.shared_key();

        let owner: Option<String> = conn
            .get(&exclusive_key)
            .await
            .wrap_err("Failed to inspect exclusive lock owner")?;
        let ttl_ms: i64 = redis::cmd("PTTL")
            .arg(&exclusive_key)
            .query_async(conn)
            .await
            .wrap_err("Failed to inspect exclusive lock TTL")?;
        let shared_count: usize = conn
            .scard(&shared_key)
            .await
            .wrap_err("Failed to inspect shared lock count")?;
        let client_id: i64 = redis::cmd("CLIENT")
            .arg("ID")
            .query_async(conn)
            .await
            .wrap_err("Failed to inspect Redis client id")?;
        let info: String = redis::cmd("INFO")
            .arg("server")
            .query_async(conn)
            .await
            .wrap_err("Failed to inspect Redis server info")?;
        let run_id = info
            .lines()
            .find_map(|line| line.strip_prefix("run_id:"))
            .unwrap_or("unknown");
        let process_id = info
            .lines()
            .find_map(|line| line.strip_prefix("process_id:"))
            .unwrap_or("unknown");
        let tcp_port = info
            .lines()
            .find_map(|line| line.strip_prefix("tcp_port:"))
            .unwrap_or("unknown");

        debug!(
            "{} for {}: exclusive_owner={:?}, exclusive_ttl_ms={}, shared_count={}, client_id={}, run_id={}, process_id={}, tcp_port={}",
            context, self.resource, owner, ttl_ms, shared_count, client_id, run_id, process_id, tcp_port
        );

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

            if tracing::enabled!(tracing::Level::DEBUG) {
                if let Err(e) =
                    Self::log_connection_identity(&mut conn, "heartbeat", Some(&resource)).await
                {
                    warn!(
                        "Failed to inspect Redis identity for heartbeat on {} (UUID: {}): {}",
                        resource, uuid, e
                    );
                }
            }

            let heartbeat_period = Duration::from_secs(HEARTBEAT_INTERVAL);
            let mut interval = tokio::time::interval_at(
                tokio::time::Instant::now() + heartbeat_period,
                heartbeat_period,
            );
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

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
                        let ttl_ms: Result<i64, _> = redis::cmd("PTTL")
                            .arg(format!("lock:{}:exclusive", resource))
                            .query_async(&mut conn)
                            .await;
                        let client_id: Result<i64, _> =
                            redis::cmd("CLIENT").arg("ID").query_async(&mut conn).await;
                        debug!(
                            "Heartbeat OK for lock {} (UUID: {}, ttl_ms={:?}, client_id={:?})",
                            resource, uuid, ttl_ms, client_id
                        );
                    }
                    Ok(0) => {
                        let owner: Result<Option<String>, _> =
                            conn.get(format!("lock:{}:exclusive", resource)).await;
                        let ttl_ms: Result<i64, _> = redis::cmd("PTTL")
                            .arg(format!("lock:{}:exclusive", resource))
                            .query_async(&mut conn)
                            .await;
                        error!(
                            "Lock lost - key expired or was removed: {} (UUID: {}, owner={:?}, ttl_ms={:?})",
                            resource, uuid, owner, ttl_ms
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
        if *self.lost_flag.lock().await {
            return Err(eyre::eyre!(
                "Lock was lost for resource: {} (UUID: {})",
                self.resource,
                self.uuid
            ));
        }

        let mut conn = self.conn.lock().await;
        let still_valid = match self.lock_type {
            Some(LockType::Exclusive) => {
                let expected_owner = self.uuid.to_string();
                let owner: Option<String> =
                    conn.get(&self.exclusive_key()).await.wrap_err_with(|| {
                        format!(
                            "Failed to verify exclusive lock ownership for {}",
                            self.resource
                        )
                    })?;

                owner.as_deref() == Some(expected_owner.as_str())
            }
            Some(LockType::Shared) => {
                let session_exists: bool =
                    conn.exists(&self.session_key()).await.wrap_err_with(|| {
                        format!("Failed to verify shared lock session for {}", self.resource)
                    })?;

                if !session_exists {
                    false
                } else {
                    conn.sismember(&self.shared_key(), self.uuid.to_string())
                        .await
                        .wrap_err_with(|| {
                            format!(
                                "Failed to verify shared lock membership for {}",
                                self.resource
                            )
                        })?
                }
            }
            None => false,
        };

        if !still_valid {
            *self.lost_flag.lock().await = true;
            self.cancellation_token.cancel();
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
        } else {
            debug!(
                "Released exclusive lock for {} during drop cleanup (UUID: {})",
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
    use std::net::{SocketAddr, ToSocketAddrs};
    use std::path::PathBuf;
    use std::sync::LazyLock;
    use test_log::test;

    fn redis_url() -> String {
        static REDIS_URL: LazyLock<String> = LazyLock::new(|| {
            let host = std::env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
            let fallback = format!("redis://{}:6379", host);

            let resolved = format!("{}:6379", host)
                .to_socket_addrs()
                .ok()
                .and_then(|mut addrs| addrs.next());

            match resolved {
                Some(SocketAddr::V4(addr)) => format!("redis://{}:{}", addr.ip(), addr.port()),
                Some(SocketAddr::V6(addr)) => {
                    format!("redis://[{}]:{}", addr.ip(), addr.port())
                }
                None => fallback,
            }
        });

        REDIS_URL.clone()
    }

    fn test_resource(name: &str) -> String {
        format!("/test/{}/{}", name, Uuid::new_v4())
    }

    #[test(tokio::test)]
    async fn test_redis_lock_new() {
        let resource = test_resource("test_new");
        let lock = PoolLockRedis::new(&redis_url(), &resource, "test_new").await;
        assert!(lock.is_ok());

        let lock = lock.unwrap();
        assert_eq!(lock.resource, resource);
        assert_eq!(
            lock.operation_name,
            LockOperation::Custom("test_new".to_string())
        );
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
        assert_eq!(
            lock.operation_name,
            LockOperation::Custom("test_path".to_string())
        );
        assert!(!lock.locked);
        assert!(lock.abort_handle.is_none());
    }

    #[test(tokio::test)]
    async fn test_redis_lock_exclusive() {
        let resource = test_resource("test_exclusive");
        let lock = PoolLockRedis::new(&redis_url(), &resource, "test_exclusive")
            .await
            .unwrap();
        let result = lock.lock_exclusive().await;

        assert!(result.is_ok());

        let locked = result.unwrap();
        assert!(locked.locked);
        assert!(locked.abort_handle.is_some());
        locked.unlock().await.unwrap();
    }

    #[test(tokio::test)]
    async fn test_redis_lock_shared() {
        let resource = test_resource("test_shared");
        let lock = PoolLockRedis::new(&redis_url(), &resource, "test_shared")
            .await
            .unwrap();
        let result = lock.lock_shared().await;

        assert!(result.is_ok());

        let locked = result.unwrap();
        assert!(locked.locked);
        assert!(locked.abort_handle.is_some());
        locked.unlock().await.unwrap();
    }

    #[test(tokio::test)]
    async fn test_redis_lock_shared_compatibility() {
        let resource = test_resource("test_shared_compat");
        let lock1 = PoolLockRedis::new(&redis_url(), &resource, "test_shared_compat")
            .await
            .unwrap();
        let locked1 = lock1.lock_shared().await.unwrap();

        // Second shared lock should succeed
        let lock2 = PoolLockRedis::new(&redis_url(), &resource, "test_shared_compat")
            .await
            .unwrap();
        let locked2 = lock2.lock_shared().await.unwrap();

        assert!(locked1.locked);
        assert!(locked2.locked);

        // Cleanup
        locked1.unlock().await.unwrap();
        locked2.unlock().await.unwrap();
    }

    #[test(tokio::test)]
    async fn test_redis_lock_exclusive_blocks_shared() {
        let resource = test_resource("test_exclusive_blocks");
        let lock1 = PoolLockRedis::new(&redis_url(), &resource, "test_exclusive_blocks")
            .await
            .unwrap();
        let locked1 = lock1.lock_exclusive().await.unwrap();

        let lock2 = PoolLockRedis::new(&redis_url(), &resource, "test_exclusive_blocks")
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
        locked1.unlock().await.unwrap();

        // Second lock should now succeed
        let locked2 = handle.await.unwrap();
        assert!(locked2.locked);
        locked2.unlock().await.unwrap();
    }

    #[test(tokio::test)]
    async fn test_redis_lock_shared_blocks_exclusive() {
        let resource = test_resource("test_shared_blocks");
        let lock1 = PoolLockRedis::new(&redis_url(), &resource, "test_shared_blocks")
            .await
            .unwrap();
        let locked1 = lock1.lock_shared().await.unwrap();

        let lock2 = PoolLockRedis::new(&redis_url(), &resource, "test_shared_blocks")
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
        locked1.unlock().await.unwrap();

        // Second lock should now succeed
        let locked2 = handle.await.unwrap();
        assert!(locked2.locked);
        locked2.unlock().await.unwrap();
    }

    #[test(tokio::test)]
    async fn test_redis_lock_heartbeat_maintains_lock() {
        let resource = test_resource("test_heartbeat");
        let lock = PoolLockRedis::new(&redis_url(), &resource, "test_heartbeat")
            .await
            .unwrap();
        let locked = lock.lock_exclusive().await.unwrap();

        // Wait longer than the TTL to ensure heartbeat is working
        tokio::time::sleep(Duration::from_secs(LOCK_TTL + 5)).await;

        // Lock should still be valid
        assert!(locked.check_valid().await.is_ok());

        // Try to acquire another lock - should fail because first lock is still alive
        let lock2 = PoolLockRedis::new(&redis_url(), &resource, "test_heartbeat")
            .await
            .unwrap();

        let handle = tokio::spawn(async move { lock2.lock_exclusive().await });

        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should still be waiting
        assert!(!handle.is_finished());

        // Cleanup
        locked.unlock().await.unwrap();
        let locked2 = handle.await.unwrap().unwrap();
        locked2.unlock().await.unwrap();
    }

    #[test(tokio::test)]
    async fn test_redis_lock_check_valid() {
        let resource = test_resource("test_check_valid");
        let lock = PoolLockRedis::new(&redis_url(), &resource, "test_check_valid")
            .await
            .unwrap();
        let locked = lock.lock_exclusive().await.unwrap();

        // Initially should be valid
        assert!(locked.check_valid().await.is_ok());

        // After some time, should still be valid (heartbeat maintains it)
        tokio::time::sleep(Duration::from_secs(5)).await;
        assert!(locked.check_valid().await.is_ok());
        locked.unlock().await.unwrap();
    }

    #[test(tokio::test)]
    async fn test_try_lock_exclusive_nowait_success() {
        let resource = test_resource("test_try_nowait_success");
        let lock = PoolLockRedis::new(&redis_url(), &resource, "test")
            .await
            .unwrap();
        let result = lock.try_lock_exclusive_nowait().await;

        assert!(result.is_ok());
        let locked = result.unwrap();
        assert!(locked.is_some());
        let locked = locked.unwrap();
        assert!(locked.locked);
        locked.unlock().await.unwrap();
    }

    #[test(tokio::test)]
    async fn test_try_lock_exclusive_nowait_fails_when_locked() {
        let resource = test_resource("test_try_nowait_fail");
        let lock1 = PoolLockRedis::new(&redis_url(), &resource, "test1")
            .await
            .unwrap();
        let locked1 = lock1.lock_exclusive().await.unwrap();

        // Try to acquire lock without waiting - should fail
        let lock2 = PoolLockRedis::new(&redis_url(), &resource, "test2")
            .await
            .unwrap();
        let result = lock2.try_lock_exclusive_nowait().await;

        assert!(result.is_ok());
        let locked2 = result.unwrap();
        assert!(locked2.is_none());
        locked1.unlock().await.unwrap();
    }

    #[test(tokio::test)]
    async fn test_try_lock_shared_wait_succeeds_with_other_shared_lock() {
        let resource = test_resource("test_try_shared_wait_success");
        let locked1 = PoolLockRedis::new(&redis_url(), &resource, "backup1")
            .await
            .unwrap()
            .lock_shared()
            .await
            .unwrap();

        let locked2 = PoolLockRedis::new(&redis_url(), &resource, "backup2")
            .await
            .unwrap()
            .try_lock_shared_wait(Duration::from_secs(1))
            .await
            .unwrap();

        assert!(locked2.is_some());

        locked1.unlock().await.unwrap();
        locked2.unwrap().unlock().await.unwrap();
    }

    #[test(tokio::test)]
    async fn test_try_lock_shared_wait_times_out_on_exclusive_lock() {
        let resource = test_resource("test_try_shared_wait_timeout");
        let locked1 = PoolLockRedis::new(&redis_url(), &resource, "fsck")
            .await
            .unwrap()
            .lock_exclusive()
            .await
            .unwrap();

        let locked2 = PoolLockRedis::new(&redis_url(), &resource, "backup")
            .await
            .unwrap()
            .try_lock_shared_wait(Duration::from_millis(200))
            .await
            .unwrap();

        assert!(locked2.is_none());

        locked1.unlock().await.unwrap();
    }

    #[test(tokio::test)]
    async fn test_has_active_lock_is_passive() {
        let resource = test_resource("test_passive_probe");

        assert!(!PoolLockRedis::has_active_lock(&redis_url(), &resource)
            .await
            .unwrap());

        let client = Client::open(redis_url()).unwrap();
        let mut conn = client.get_multiplexed_async_connection().await.unwrap();
        let owner: Option<String> = conn
            .get(format!("lock:{}:exclusive", resource))
            .await
            .unwrap();

        assert!(owner.is_none());
    }

    #[test(tokio::test)]
    async fn test_has_active_lock_detects_exclusive_lock() {
        let resource = test_resource("test_passive_probe_exclusive");
        let locked = PoolLockRedis::new(&redis_url(), &resource, "test_probe_exclusive")
            .await
            .unwrap()
            .lock_exclusive()
            .await
            .unwrap();

        assert!(PoolLockRedis::has_active_lock(&redis_url(), &resource)
            .await
            .unwrap());

        locked.unlock().await.unwrap();
    }

    #[test(tokio::test)]
    async fn test_has_active_lock_detects_shared_lock() {
        let resource = test_resource("test_passive_probe_shared");
        let locked = PoolLockRedis::new(&redis_url(), &resource, "test_probe_shared")
            .await
            .unwrap()
            .lock_shared()
            .await
            .unwrap();

        assert!(PoolLockRedis::has_active_lock(&redis_url(), &resource)
            .await
            .unwrap());

        locked.unlock().await.unwrap();
    }

    #[test(tokio::test)]
    async fn test_active_exclusive_lock_returns_none_when_unlocked() {
        let resource = test_resource("test_active_exclusive_none");

        let active = PoolLockRedis::active_exclusive_lock(&redis_url(), &resource)
            .await
            .unwrap();

        assert!(active.is_none());
    }

    #[test(tokio::test)]
    async fn test_active_exclusive_lock_reports_operation_name() {
        let resource = test_resource("test_active_exclusive_operation");
        let locked = PoolLockRedis::new(&redis_url(), &resource, "fsck")
            .await
            .unwrap()
            .lock_exclusive()
            .await
            .unwrap();

        let active = PoolLockRedis::active_exclusive_lock(&redis_url(), &resource)
            .await
            .unwrap();

        assert_eq!(
            active,
            Some(ActiveExclusiveLock {
                operation_name: Some(LockOperation::Pool(PoolLockOperation::Fsck)),
            })
        );

        locked.unlock().await.unwrap();
    }
}

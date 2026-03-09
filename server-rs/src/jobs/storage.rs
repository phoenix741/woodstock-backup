use apalis_redis::{connect, Config as RedisConfig, RedisStorage};
use eyre::Result;
use std::time::Duration;

use crate::jobs::types::{
    BackupQueueJob, MaintenanceJobData, QueueName, RestoreJobData, ScheduleQueueJob,
};

/// Heartbeat interval for all queues.
///
/// Apalis refreshes the consumer's timestamp in Redis every `KEEP_ALIVE` seconds.
/// A consumer whose timestamp is older than `ORPHAN_TIMEOUT_*` is considered dead and
/// its in-flight jobs are re-enqueued. Keeping this interval short ensures that even
/// under heavy Tokio load (e.g. during a large backup or fsck), the consumer stays
/// registered.
const KEEP_ALIVE: Duration = Duration::from_secs(10);

/// Orphan timeout for quick queues (schedule).
///
/// Schedule jobs only dispatch work; they complete quickly and can safely be
/// re-enqueued after a short delay if the worker crashes.
const ORPHAN_TIMEOUT_QUICK: Duration = Duration::from_secs(300);

/// Orphan timeout for long-running jobs (backup, restore).
///
/// Set to 1 hour. Backup/restore jobs stream large datasets over gRPC and can
/// run for many hours; 1h gives enough slack that a healthy job is never
/// re-enqueued, while still recovering quickly after a real worker crash.
const ORPHAN_TIMEOUT_LONG: Duration = Duration::from_secs(3_600); // 1 hour

/// Orphan timeout for maintenance jobs (fsck, cleanup_refcnt).
///
/// Set to 12 hours, much longer than `ORPHAN_TIMEOUT_LONG`.
/// Rationale: maintenance jobs (especially fsck) can be re-run safely (idempotent),
/// but we do NOT want Apalis to re-enqueue a fsck job just because the host went to
/// sleep for an hour and the `KEEP_ALIVE` heartbeat stopped.  A sleep of less than
/// 12 h will therefore not produce a duplicate fsck run.
///
/// If the worker truly crashes, Apalis will still recover the job within 12 h,
/// which is acceptable for a background maintenance task.  Meanwhile the
/// `CancellationToken` in `PoolLockRedis` will abort any concurrent run that
/// detects the Redis lock was lost, preventing double execution in the rare case
/// where a re-enqueue does occur.
const ORPHAN_TIMEOUT_MAINTENANCE: Duration = Duration::from_secs(43_200); // 12 hours

#[derive(Clone)]
pub struct ApalisRedisStorage {
    pub schedule_storage: RedisStorage<ScheduleQueueJob>,
    pub backup_storage: RedisStorage<BackupQueueJob>,
    pub interactive_storage: RedisStorage<RestoreJobData>,
    pub maintenance_storage: RedisStorage<MaintenanceJobData>,
}

impl ApalisRedisStorage {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let conn_schedule = connect(redis_url).await?;
        let conn_backup = connect(redis_url).await?;
        let conn_interactive = connect(redis_url).await?;
        let conn_maintenance = connect(redis_url).await?;

        // Schedule jobs are quick dispatchers — short orphan timeout is fine.
        let schedule_storage: RedisStorage<ScheduleQueueJob> = RedisStorage::new_with_config(
            conn_schedule,
            RedisConfig::default()
                .set_namespace(QueueName::Schedule.as_str())
                .set_keep_alive(KEEP_ALIVE)
                .set_reenqueue_orphaned_after(ORPHAN_TIMEOUT_QUICK),
        );
        // Backup jobs stream large amounts of data over gRPC and can run for many hours.
        // The orphan timeout is aligned with DEFAULT_BACKUP_TIMEOUT_SECS to prevent
        // Apalis from re-enqueuing a job that is still actively transferring data.
        let backup_storage: RedisStorage<BackupQueueJob> = RedisStorage::new_with_config(
            conn_backup,
            RedisConfig::default()
                .set_namespace(QueueName::Backup.as_str())
                .set_keep_alive(KEEP_ALIVE)
                .set_reenqueue_orphaned_after(ORPHAN_TIMEOUT_LONG),
        );
        // Restore jobs also stream large datasets over gRPC — same timeout as backup.
        let interactive_storage: RedisStorage<RestoreJobData> = RedisStorage::new_with_config(
            conn_interactive,
            RedisConfig::default()
                .set_namespace(QueueName::Interactive.as_str())
                .set_keep_alive(KEEP_ALIVE)
                .set_reenqueue_orphaned_after(ORPHAN_TIMEOUT_LONG),
        );
        // Maintenance jobs (fsck, cleanup_refcnt) scan the full pool and can run for hours.
        // We use ORPHAN_TIMEOUT_MAINTENANCE (12h) instead of ORPHAN_TIMEOUT_LONG (1h) so that
        // a host suspended for less than 12h does not trigger a duplicate fsck run.
        // A CancellationToken in PoolLockRedis provides a second line of defence if a
        // re-enqueue does occur.
        let maintenance_storage: RedisStorage<MaintenanceJobData> = RedisStorage::new_with_config(
            conn_maintenance,
            RedisConfig::default()
                .set_namespace(QueueName::Maintenance.as_str())
                .set_keep_alive(KEEP_ALIVE)
                .set_reenqueue_orphaned_after(ORPHAN_TIMEOUT_MAINTENANCE),
        );

        Ok(Self {
            schedule_storage,
            backup_storage,
            interactive_storage,
            maintenance_storage,
        })
    }
}

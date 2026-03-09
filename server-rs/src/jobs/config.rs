//! Configuration spécifique au binaire job_worker (concurrence, retries, TTL etc.)
//!
//! Variables supportées:
//! - BACKUP_CONCURRENCY (usize, défaut 2)
//! - RESTORE_CONCURRENCY (usize, défaut 8)
//! - MAINTENANCE_CONCURRENCY (usize, défaut 2)
//! - PROGRESS_SNAPSHOT_TTL (u64 secondes, défaut 86400)
//! - REDIS_URL (optionnel, override redis_url global)
//! - HOST_LOCK_TTL_MS (u64, défaut 60000)

use std::env;

#[derive(Debug, Clone)]
pub struct JobWorkerConfig {
    pub backup_concurrency: usize,
    pub restore_concurrency: usize,
    pub maintenance_concurrency: usize,
    pub progress_snapshot_ttl_sec: i64,
    pub host_lock_ttl_ms: u64,
}

impl Default for JobWorkerConfig {
    fn default() -> Self {
        Self {
            backup_concurrency: env::var("BACKUP_CONCURRENCY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2)
                .max(1),
            restore_concurrency: env::var("RESTORE_CONCURRENCY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8)
                .max(1),
            maintenance_concurrency: env::var("MAINTENANCE_CONCURRENCY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2)
                .max(1),
            progress_snapshot_ttl_sec: env::var("PROGRESS_SNAPSHOT_TTL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(24 * 3600),
            host_lock_ttl_ms: env::var("HOST_LOCK_TTL_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60_000),
        }
    }
}

impl JobWorkerConfig {
    pub fn from_env() -> Self {
        Self::default()
    }
}

//! Redis-backed registry of running server-side services (`api_server`, `client_api_server`,
//! `scheduler`, `job_worker`). Each process registers itself under `service:{service_type}:{uuid}`
//! and refreshes a TTL via a background heartbeat, so a crashed or killed process disappears from
//! the registry on its own without any explicit deregistration — mirrors the hash+TTL+heartbeat
//! pattern used for distributed locks in [`crate::utils::lock_redis`].

use eyre::{Result, WrapErr};
use redis::{AsyncCommands, AsyncIter, Client};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::warn;
use uuid::Uuid;

/// Interval in seconds between registry heartbeat refreshes.
const HEARTBEAT_INTERVAL: u64 = 10;
/// TTL in seconds for a service registration. A process that stops heartbeating
/// (crash, kill -9, graceful shutdown) disappears from the registry within this delay.
const SERVICE_TTL: u64 = 30;

/// A single registered service instance, as read back from Redis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceInfo {
    pub service_type: String,
    pub instance_id: String,
    pub version: String,
    pub hostname: String,
    pub pid: u32,
    pub started_at: u64,
}

fn registry_key(service_type: &str, instance_id: &Uuid) -> String {
    format!("service:{service_type}:{instance_id}")
}

/// Registers this process in the service registry and spawns a background task that
/// keeps the registration alive for as long as the process runs.
///
/// Non-blocking by design: registration failures (Redis unreachable at startup) are logged
/// and retried on the next heartbeat tick, never propagated to the caller — a binary must be
/// able to start serving its actual purpose even if the registry is temporarily unavailable.
///
/// `version` must be the *caller's* crate version (`env!("CARGO_PKG_VERSION")` evaluated in the
/// calling crate) — evaluating that macro inside this function would report `woodstock-rs`'s own
/// version instead of the caller's.
pub fn register_service(redis_url: String, service_type: &'static str, version: String) {
    let instance_id = Uuid::new_v4();
    let key = registry_key(service_type, &instance_id);
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let pid = std::process::id();
    let started_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    tokio::spawn(async move {
        let Ok(client) = Client::open(redis_url.as_str()) else {
            warn!(
                "Service registry: invalid Redis URL, giving up registering {} ({})",
                service_type, key
            );
            return;
        };

        let mut interval = tokio::time::interval(Duration::from_secs(HEARTBEAT_INTERVAL));
        loop {
            interval.tick().await;

            let write = async {
                let mut conn = client.get_multiplexed_async_connection().await?;
                redis::pipe()
                    .atomic()
                    .hset(&key, "service_type", service_type)
                    .ignore()
                    .hset(&key, "instance_id", instance_id.to_string())
                    .ignore()
                    .hset(&key, "version", version.as_str())
                    .ignore()
                    .hset(&key, "hostname", hostname.as_str())
                    .ignore()
                    .hset(&key, "pid", pid)
                    .ignore()
                    .hset(&key, "started_at", started_at)
                    .ignore()
                    .expire(&key, SERVICE_TTL as i64)
                    .ignore()
                    .query_async::<()>(&mut conn)
                    .await
            }
            .await;

            if let Err(err) = write {
                warn!(
                    "Service registry: failed to refresh registration for {} ({}): {}",
                    service_type, key, err
                );
            }
        }
    });
}

/// Lists all services currently registered (i.e. whose heartbeat has not expired).
///
/// Uses `SCAN` rather than `KEYS` so this never blocks the Redis event loop, matching every
/// other registry/cache scan in this codebase (see `ProgressReader::list` in server-rs).
pub async fn list_services(redis_url: &str) -> Result<Vec<ServiceInfo>> {
    let client = Client::open(redis_url).wrap_err("Failed to open Redis connection")?;
    let mut conn = client
        .get_multiplexed_async_connection()
        .await
        .wrap_err("Failed to connect to Redis")?;

    let mut keys: Vec<String> = Vec::new();
    let mut iter: AsyncIter<'_, String> = conn
        .scan_match("service:*")
        .await
        .wrap_err("Failed to scan service registry keys")?;
    while let Some(key) = iter.next_item().await {
        match key {
            Ok(key) => keys.push(key),
            Err(err) => warn!("Service registry: scan failed on a key: {}", err),
        }
    }
    drop(iter);

    let mut services = Vec::with_capacity(keys.len());
    for key in keys {
        let fields: HashMap<String, String> = conn
            .hgetall(&key)
            .await
            .wrap_err_with(|| format!("Failed to read service registry entry {key}"))?;

        let Some(service_type) = fields.get("service_type").cloned() else {
            continue;
        };
        let Some(instance_id) = fields.get("instance_id").cloned() else {
            continue;
        };
        let version = fields.get("version").cloned().unwrap_or_default();
        let hostname = fields.get("hostname").cloned().unwrap_or_default();
        let pid = fields.get("pid").and_then(|v| v.parse().ok()).unwrap_or(0);
        let started_at = fields
            .get("started_at")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        services.push(ServiceInfo {
            service_type,
            instance_id,
            version,
            hostname,
            pid,
            started_at,
        });
    }

    Ok(services)
}

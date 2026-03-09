//! Generic Redis read-through cache helpers.
//!
//! # Overview
//!
//! This module provides two small utilities to add a Redis cache layer in front
//! of any async disk-reading operation:
//!
//! - [`cache_wrap`]: Read-through helper. Returns the cached value on HIT; on
//!   MISS calls a user-supplied async closure (typically a disk read), caches
//!   the result with a TTL via `SETEX`, then returns it.
//! - [`cache_invalidate`]: Deletes a single key from Redis.
//!
//! All Redis errors are **traced** but never propagated — the helpers are designed
//! to be used in a "best-effort" fashion: a Redis outage falls back transparently
//! to the source (disk).
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{de::DeserializeOwned, Serialize};
use std::{future::Future, sync::Arc};
use tokio::sync::Mutex;
use tracing::{debug, warn};

/// Reads `key` from Redis and deserializes the cached value on a HIT.
///
/// On a MISS (key missing or deserialization error), calls the user-supplied
/// async `fetch` closure, stores the result via `SETEX` with `ttl_secs`, and
/// returns it.
///
/// # Arguments
///
/// * `conn` – Shared Redis connection.
/// * `key` – Cache key.
/// * `ttl_secs` – Time-to-live in seconds for fresh cache entries.
/// * `fetch` – Async closure invoked on cache miss; must return `T`.
///
/// # Behaviour on errors
///
/// Redis errors (GET, SETEX, serialization) are traced but never propagated.
/// The function always returns the value produced by `fetch` in the worst case.
pub async fn cache_wrap<T, F, Fut>(
    conn: &Arc<Mutex<ConnectionManager>>,
    key: &str,
    ttl_secs: u64,
    fetch: F,
) -> T
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
{
    // ── Cache GET ──────────────────────────────────────────────────────────
    {
        let mut guard = conn.lock().await;
        match guard.get::<_, Option<String>>(key).await {
            Ok(Some(raw)) => match serde_json::from_str::<T>(&raw) {
                Ok(value) => {
                    debug!(key, "Cache HIT");
                    return value;
                }
                Err(e) => {
                    warn!(key, error = %e, "Cache deserialization error, fetching from source")
                }
            },
            Ok(None) => debug!(key, "Cache MISS"),
            Err(e) => warn!(key, error = %e, "Redis GET error, fetching from source"),
        }
    }

    // ── Fetch from source ──────────────────────────────────────────────────
    let value = fetch().await;

    // ── Cache SETEX (best-effort) ──────────────────────────────────────────
    match serde_json::to_string(&value) {
        Ok(json) => {
            let mut guard = conn.lock().await;
            if let Err(e) = guard.set_ex::<_, _, ()>(key, json, ttl_secs).await {
                warn!(key, error = %e, "Redis SETEX error");
            }
        }
        Err(e) => warn!(key, error = %e, "Cache serialization error"),
    }

    value
}

/// Deletes `key` from Redis (cache invalidation).
///
/// Errors are traced but never propagated.
pub async fn cache_invalidate(conn: &Arc<Mutex<ConnectionManager>>, key: &str) {
    let mut guard = conn.lock().await;
    if let Err(e) = guard.del::<_, ()>(key).await {
        warn!(key, error = %e, "Redis DEL error during cache invalidation");
    } else {
        debug!(key, "Cache key invalidated");
    }
}

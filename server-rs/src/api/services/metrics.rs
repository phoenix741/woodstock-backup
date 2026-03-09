//! Metrics service for Prometheus integration
//!
//! Provides the exact same metrics as the NestJS PrometheusService

use eyre::Result;
use prometheus::{Encoder, Gauge, GaugeVec, Opts, Registry, TextEncoder};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Cache structure matching NestJS implementation
#[derive(Debug, Clone)]
struct MetricsCache {
    ts: Instant,
    disk: DiskUsage,
    hosts: HashMap<String, HostStats>,
    pool: PoolStats,
    queue: QueueStats,
}

/// Disk usage statistics
#[derive(Debug, Clone)]
struct DiskUsage {
    size: u64,
    used: u64,
    free: u64,
}

/// Host-specific backup statistics
#[derive(Debug, Clone)]
struct HostStats {
    last_backup_size: u64,
    last_backup_time: i64,
    last_backup_age: i64,
    last_backup_duration: i64,
    last_backup_complete: i32,
    longest_chain: i64,
    nb_chunk: i64,
    nb_ref: i64,
    size: u64,
    compressed_size: u64,
    backup_count: i64,
}

/// Pool-wide statistics
#[derive(Debug, Clone)]
struct PoolStats {
    longest_chain: i64,
    nb_chunk: i64,
    nb_ref: i64,
    size: u64,
    compressed_size: u64,
    unused_size: u64,
}

/// Queue statistics
#[derive(Debug, Clone)]
struct QueueStats {
    completed: i64,
    failed: i64,
    delayed: i64,
    active: i64,
    waiting: i64,
}

/// Global Prometheus registry
pub static REGISTRY: LazyLock<Registry> = LazyLock::new(Registry::new);

// Pool metrics
/// Total disk space available on the pool directory in Mo
pub static POOL_TOTAL_DISK_SPACE: LazyLock<Gauge> = LazyLock::new(|| {
    Gauge::with_opts(Opts::new(
        "pool_total_disk_space",
        "Total disk space available on the pool directory in Mo",
    ))
    // SAFETY: hardcoded metric options are always valid
    .expect("hardcoded Prometheus metric POOL_TOTAL_DISK_SPACE is always valid")
});

/// Total disk space free on the pool directory in Mo
pub static POOL_TOTAL_FREE_SPACE: LazyLock<Gauge> = LazyLock::new(|| {
    Gauge::with_opts(Opts::new(
        "pool_total_free_space",
        "Total disk space free on the pool directory in Mo",
    ))
    .expect("hardcoded Prometheus metric POOL_TOTAL_FREE_SPACE is always valid")
});

/// Total disk space used on the pool directory in Mo
pub static POOL_TOTAL_USED_SPACE: LazyLock<Gauge> = LazyLock::new(|| {
    Gauge::with_opts(Opts::new(
        "pool_total_used_space",
        "Total disk space used on the pool directory in Mo",
    ))
    .expect("hardcoded Prometheus metric POOL_TOTAL_USED_SPACE is always valid")
});

/// Pool Longest backup chain
pub static POOL_LONGEST_CHAIN: LazyLock<Gauge> = LazyLock::new(|| {
    Gauge::with_opts(Opts::new("pool_longest_chain", "Pool Longest backup chain"))
        .expect("hardcoded Prometheus metric POOL_LONGEST_CHAIN is always valid")
});

/// Number of chunks in the pool
pub static POOL_NB_CHUNK: LazyLock<Gauge> = LazyLock::new(|| {
    Gauge::with_opts(Opts::new("pool_nb_chunk", "Number of chunks in the pool"))
        .expect("hardcoded Prometheus metric POOL_NB_CHUNK is always valid")
});

/// Number of references in the pool
pub static POOL_NB_REF: LazyLock<Gauge> = LazyLock::new(|| {
    Gauge::with_opts(Opts::new("pool_nb_ref", "Number of references in the pool"))
        .expect("hardcoded Prometheus metric POOL_NB_REF is always valid")
});

/// Pool Size of the pool in Mo
pub static POOL_SIZE: LazyLock<Gauge> = LazyLock::new(|| {
    Gauge::with_opts(Opts::new("pool_size", "Pool Size of the pool in Mo"))
        .expect("hardcoded Prometheus metric POOL_SIZE is always valid")
});

/// Compressed pool size of the pool in Mo
pub static POOL_COMPRESSED_SIZE: LazyLock<Gauge> = LazyLock::new(|| {
    Gauge::with_opts(Opts::new(
        "pool_compressed_size",
        "Compressed pool size of the pool in Mo",
    ))
    .expect("hardcoded Prometheus metric POOL_COMPRESSED_SIZE is always valid")
});

/// Content of the pool that is not used in Mo
pub static POOL_UNUSED_SIZE: LazyLock<Gauge> = LazyLock::new(|| {
    Gauge::with_opts(Opts::new(
        "pool_unusedSize",
        "Content of the pool that is not used in Mo",
    ))
    .expect("hardcoded Prometheus metric POOL_UNUSED_SIZE is always valid")
});

// Host metrics (with host label)
/// Size of last backup in Mo
pub static HOST_BACKUP_LAST_BACKUP_SIZE: LazyLock<GaugeVec> = LazyLock::new(|| {
    GaugeVec::new(
        Opts::new("host_backup_last_backup_size", "Size of last backup in Mo"),
        &["host"],
    )
    .expect("hardcoded Prometheus metric HOST_BACKUP_LAST_BACKUP_SIZE is always valid")
});

/// Time since last backup
pub static HOST_BACKUP_LAST_BACKUP_TIME: LazyLock<GaugeVec> = LazyLock::new(|| {
    GaugeVec::new(
        Opts::new("host_backup_last_backup_time", "Time since last backup"),
        &["host"],
    )
    .expect("hardcoded Prometheus metric HOST_BACKUP_LAST_BACKUP_TIME is always valid")
});

/// Time of last backup
pub static HOST_BACKUP_LAST_BACKUP_AGE: LazyLock<GaugeVec> = LazyLock::new(|| {
    GaugeVec::new(
        Opts::new("host_backup_last_backup_age", "Time of last backup"),
        &["host"],
    )
    .expect("hardcoded Prometheus metric HOST_BACKUP_LAST_BACKUP_AGE is always valid")
});

/// Duration of last backup
pub static HOST_BACKUP_LAST_BACKUP_DURATION: LazyLock<GaugeVec> = LazyLock::new(|| {
    GaugeVec::new(
        Opts::new(
            "host_backup_last_backup_duration",
            "Duration of last backup",
        ),
        &["host"],
    )
    .expect("hardcoded Prometheus metric HOST_BACKUP_LAST_BACKUP_DURATION is always valid")
});

/// Is last backup completed
pub static HOST_BACKUP_LAST_BACKUP_COMPLETED: LazyLock<GaugeVec> = LazyLock::new(|| {
    GaugeVec::new(
        Opts::new(
            "host_backup_last_backup_completed",
            "Is last backup completed",
        ),
        &["host"],
    )
    .expect("hardcoded Prometheus metric HOST_BACKUP_LAST_BACKUP_COMPLETED is always valid")
});

/// Longest backup chain
pub static HOST_BACKUP_LONGEST_CHAIN: LazyLock<GaugeVec> = LazyLock::new(|| {
    GaugeVec::new(
        Opts::new("host_backup_longest_chain", "Longest backup chain"),
        &["host"],
    )
    .expect("hardcoded Prometheus metric HOST_BACKUP_LONGEST_CHAIN is always valid")
});

/// Number of chunks for the host
pub static HOST_BACKUP_NB_CHUNK: LazyLock<GaugeVec> = LazyLock::new(|| {
    GaugeVec::new(
        Opts::new("host_backup_nb_chunk", "Number of chunks for the host"),
        &["host"],
    )
    .expect("hardcoded Prometheus metric HOST_BACKUP_NB_CHUNK is always valid")
});

/// Number of references for the host
pub static HOST_BACKUP_NB_REF: LazyLock<GaugeVec> = LazyLock::new(|| {
    GaugeVec::new(
        Opts::new("host_backup_nb_ref", "Number of references for the host"),
        &["host"],
    )
    .expect("hardcoded Prometheus metric HOST_BACKUP_NB_REF is always valid")
});

/// Pool Size of the backup in Mo
pub static HOST_BACKUP_SIZE: LazyLock<GaugeVec> = LazyLock::new(|| {
    GaugeVec::new(
        Opts::new("host_backup_size", "Pool Size of the backup in Mo"),
        &["host"],
    )
    .expect("hardcoded Prometheus metric HOST_BACKUP_SIZE is always valid")
});

/// Compressed pool size of the backup in Mo
pub static HOST_BACKUP_COMPRESSED_SIZE: LazyLock<GaugeVec> = LazyLock::new(|| {
    GaugeVec::new(
        Opts::new(
            "host_backup_compressed_size",
            "Compressed pool size of the backup in Mo",
        ),
        &["host"],
    )
    .expect("hardcoded Prometheus metric HOST_BACKUP_COMPRESSED_SIZE is always valid")
});

/// Number of backups for a host
pub static HOST_BACKUP_COUNT: LazyLock<GaugeVec> = LazyLock::new(|| {
    GaugeVec::new(
        Opts::new("host_backup_count", "Number of backups for a host"),
        &["host"],
    )
    .expect("hardcoded Prometheus metric HOST_BACKUP_COUNT is always valid")
});

// Queue metrics
/// Number of job completed
pub static JOBS_COMPLETED_TOTAL: LazyLock<Gauge> = LazyLock::new(|| {
    Gauge::with_opts(Opts::new("jobs_completed_total", "Number of job completed"))
        .expect("hardcoded Prometheus metric JOBS_COMPLETED_TOTAL is always valid")
});

/// Number of job failed
pub static JOBS_FAILED_TOTAL: LazyLock<Gauge> = LazyLock::new(|| {
    Gauge::with_opts(Opts::new("jobs_failed_total", "Number of job failed"))
        .expect("hardcoded Prometheus metric JOBS_FAILED_TOTAL is always valid")
});

/// Number of job active
pub static JOBS_ACTIVE_TOTAL: LazyLock<Gauge> = LazyLock::new(|| {
    Gauge::with_opts(Opts::new("jobs_active_total", "Number of job active"))
        .expect("hardcoded Prometheus metric JOBS_ACTIVE_TOTAL is always valid")
});

/// Number of job delayed
pub static JOBS_DELAYED_TOTAL: LazyLock<Gauge> = LazyLock::new(|| {
    Gauge::with_opts(Opts::new("jobs_delayed_total", "Number of job delayed"))
        .expect("hardcoded Prometheus metric JOBS_DELAYED_TOTAL is always valid")
});

/// Number of job waiting
pub static JOBS_WAITING_TOTAL: LazyLock<Gauge> = LazyLock::new(|| {
    Gauge::with_opts(Opts::new("jobs_waiting_total", "Number of job waiting"))
        .expect("hardcoded Prometheus metric JOBS_WAITING_TOTAL is always valid")
});

/// Initialize all metrics in the registry
pub fn init_metrics() -> Result<()> {
    // Pool metrics
    REGISTRY.register(Box::new(POOL_TOTAL_DISK_SPACE.clone()))?;
    REGISTRY.register(Box::new(POOL_TOTAL_FREE_SPACE.clone()))?;
    REGISTRY.register(Box::new(POOL_TOTAL_USED_SPACE.clone()))?;
    REGISTRY.register(Box::new(POOL_LONGEST_CHAIN.clone()))?;
    REGISTRY.register(Box::new(POOL_NB_CHUNK.clone()))?;
    REGISTRY.register(Box::new(POOL_NB_REF.clone()))?;
    REGISTRY.register(Box::new(POOL_SIZE.clone()))?;
    REGISTRY.register(Box::new(POOL_COMPRESSED_SIZE.clone()))?;
    REGISTRY.register(Box::new(POOL_UNUSED_SIZE.clone()))?;

    // Host metrics
    REGISTRY.register(Box::new(HOST_BACKUP_LAST_BACKUP_SIZE.clone()))?;
    REGISTRY.register(Box::new(HOST_BACKUP_LAST_BACKUP_TIME.clone()))?;
    REGISTRY.register(Box::new(HOST_BACKUP_LAST_BACKUP_AGE.clone()))?;
    REGISTRY.register(Box::new(HOST_BACKUP_LAST_BACKUP_DURATION.clone()))?;
    REGISTRY.register(Box::new(HOST_BACKUP_LAST_BACKUP_COMPLETED.clone()))?;
    REGISTRY.register(Box::new(HOST_BACKUP_LONGEST_CHAIN.clone()))?;
    REGISTRY.register(Box::new(HOST_BACKUP_NB_CHUNK.clone()))?;
    REGISTRY.register(Box::new(HOST_BACKUP_NB_REF.clone()))?;
    REGISTRY.register(Box::new(HOST_BACKUP_SIZE.clone()))?;
    REGISTRY.register(Box::new(HOST_BACKUP_COMPRESSED_SIZE.clone()))?;
    REGISTRY.register(Box::new(HOST_BACKUP_COUNT.clone()))?;

    // Queue metrics
    REGISTRY.register(Box::new(JOBS_COMPLETED_TOTAL.clone()))?;
    REGISTRY.register(Box::new(JOBS_FAILED_TOTAL.clone()))?;
    REGISTRY.register(Box::new(JOBS_ACTIVE_TOTAL.clone()))?;
    REGISTRY.register(Box::new(JOBS_DELAYED_TOTAL.clone()))?;
    REGISTRY.register(Box::new(JOBS_WAITING_TOTAL.clone()))?;

    Ok(())
}

/// Metrics service for collecting and exposing Prometheus metrics
/// Implements the exact same caching and collection logic as NestJS PrometheusService
pub struct MetricsService {
    encoder: TextEncoder,
    cache: Arc<RwLock<Option<MetricsCache>>>,
}

impl MetricsService {
    /// Create new MetricsService instance
    pub fn new() -> Self {
        Self {
            encoder: TextEncoder::new(),
            cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Get all metrics in Prometheus text format
    pub async fn get_metrics(&self) -> Result<String> {
        // Update metrics from cache
        self.update_metrics_from_cache().await?;

        let metric_families = REGISTRY.gather();
        let mut buffer = Vec::new();
        self.encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    /// Update metrics from cached data (with 60s TTL like NestJS)
    async fn update_metrics_from_cache(&self) -> Result<()> {
        let cache_valid = {
            let cache_read = self.cache.read().await;
            if let Some(cache) = cache_read.as_ref() {
                cache.ts.elapsed() <= Duration::from_secs(60)
            } else {
                false
            }
        };

        if !cache_valid {
            // Refresh cache
            let new_cache = self.collect_metrics_data().await?;
            {
                let mut cache_write = self.cache.write().await;
                *cache_write = Some(new_cache.clone());
            }
            self.update_prometheus_metrics(&new_cache).await?;
        } else {
            // Use existing cache
            let cache_read = self.cache.read().await;
            if let Some(cache) = cache_read.as_ref() {
                self.update_prometheus_metrics(cache).await?;
            }
        }

        Ok(())
    }

    /// Collect fresh metrics data from woodstock-rs services
    async fn collect_metrics_data(&self) -> Result<MetricsCache> {
        // TODO: Integrate with woodstock-rs services to get real data
        // For now, return placeholder data matching the NestJS structure

        let disk = DiskUsage {
            size: 1000000000000, // 1TB
            used: 500000000000,  // 500GB
            free: 500000000000,  // 500GB
        };

        let mut hosts = HashMap::new();
        // TODO: Get real hosts from woodstock-rs HostsService
        hosts.insert(
            "example-host".to_string(),
            HostStats {
                last_backup_size: 1000000000, // 1GB
                last_backup_time: 1627849200,
                last_backup_age: 3600,
                last_backup_duration: 1800,
                last_backup_complete: 1,
                longest_chain: 10,
                nb_chunk: 1000,
                nb_ref: 500,
                size: 5000000000,            // 5GB
                compressed_size: 2500000000, // 2.5GB
                backup_count: 15,
            },
        );

        let pool = PoolStats {
            longest_chain: 20,
            nb_chunk: 10000,
            nb_ref: 5000,
            size: 50000000000,            // 50GB
            compressed_size: 25000000000, // 25GB
            unused_size: 1000000000,      // 1GB
        };

        let queue = QueueStats {
            completed: 100,
            failed: 5,
            delayed: 2,
            active: 1,
            waiting: 3,
        };

        Ok(MetricsCache {
            ts: Instant::now(),
            disk,
            hosts,
            pool,
            queue,
        })
    }

    /// Update Prometheus metrics from cached data
    async fn update_prometheus_metrics(&self, cache: &MetricsCache) -> Result<()> {
        // Update pool disk metrics (convert bytes to Mo)
        POOL_TOTAL_DISK_SPACE.set((cache.disk.size / 1024 / 1024) as f64);
        POOL_TOTAL_FREE_SPACE.set((cache.disk.free / 1024 / 1024) as f64);
        POOL_TOTAL_USED_SPACE.set((cache.disk.used / 1024 / 1024) as f64);

        // Update pool statistics
        POOL_LONGEST_CHAIN.set(cache.pool.longest_chain as f64);
        POOL_NB_CHUNK.set(cache.pool.nb_chunk as f64);
        POOL_NB_REF.set(cache.pool.nb_ref as f64);
        POOL_SIZE.set((cache.pool.size / 1024 / 1024) as f64);
        POOL_COMPRESSED_SIZE.set((cache.pool.compressed_size / 1024 / 1024) as f64);
        POOL_UNUSED_SIZE.set((cache.pool.unused_size / 1024 / 1024) as f64);

        // Update host-specific metrics
        for (host, stats) in &cache.hosts {
            let host_label = &[host.as_str()];

            HOST_BACKUP_LAST_BACKUP_SIZE
                .with_label_values(host_label)
                .set((stats.last_backup_size / 1024 / 1024) as f64);
            HOST_BACKUP_LAST_BACKUP_TIME
                .with_label_values(host_label)
                .set(stats.last_backup_time as f64);
            HOST_BACKUP_LAST_BACKUP_AGE
                .with_label_values(host_label)
                .set(stats.last_backup_age as f64);
            HOST_BACKUP_LAST_BACKUP_DURATION
                .with_label_values(host_label)
                .set(stats.last_backup_duration as f64);
            HOST_BACKUP_LAST_BACKUP_COMPLETED
                .with_label_values(host_label)
                .set(stats.last_backup_complete as f64);
            HOST_BACKUP_LONGEST_CHAIN
                .with_label_values(host_label)
                .set(stats.longest_chain as f64);
            HOST_BACKUP_NB_CHUNK
                .with_label_values(host_label)
                .set(stats.nb_chunk as f64);
            HOST_BACKUP_NB_REF
                .with_label_values(host_label)
                .set(stats.nb_ref as f64);
            HOST_BACKUP_SIZE
                .with_label_values(host_label)
                .set((stats.size / 1024 / 1024) as f64);
            HOST_BACKUP_COMPRESSED_SIZE
                .with_label_values(host_label)
                .set((stats.compressed_size / 1024 / 1024) as f64);
            HOST_BACKUP_COUNT
                .with_label_values(host_label)
                .set(stats.backup_count as f64);
        }

        // Update queue metrics
        JOBS_COMPLETED_TOTAL.set(cache.queue.completed as f64);
        JOBS_FAILED_TOTAL.set(cache.queue.failed as f64);
        JOBS_ACTIVE_TOTAL.set(cache.queue.active as f64);
        JOBS_DELAYED_TOTAL.set(cache.queue.delayed as f64);
        JOBS_WAITING_TOTAL.set(cache.queue.waiting as f64);

        Ok(())
    }
}

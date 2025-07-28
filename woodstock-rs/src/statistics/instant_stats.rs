//! Construction de statistiques instantanées (agrégées) pour exposition API.
use std::time::{SystemTime, UNIX_EPOCH};

use super::{HostStatsUsage, PoolStatistics, StatsDiskUsage};
use eyre::Result;
use std::path::Path;
use sysinfo::Disks;

/// Construit un HostStatsUsage à partir d'un PoolStatistics et de métadonnées backup.
/// Pour l'instant nous ne disposons pas encore des infos de dernier backup; on place des valeurs neutres.
pub fn to_host_usage(stats: &PoolStatistics) -> HostStatsUsage {
    let mut usage = HostStatsUsage::new(
        stats.longest_chain,
        stats.nb_chunk,
        stats.nb_ref,
        stats.size,
        stats.compressed_size,
        stats.unused_size,
    );
    usage.last_backup_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    usage
}

pub fn get_space<P: AsRef<Path>>(pool_path: P) -> Result<StatsDiskUsage> {
    let mut disks = Disks::new_with_refreshed_list();
    disks.sort_by(|a, b| {
        a.mount_point()
            .components()
            .count()
            .cmp(&b.mount_point().components().count())
    });

    let pool_path = pool_path.as_ref().canonicalize()?;

    let disk = disks
        .iter()
        .find(|d| pool_path.starts_with(d.mount_point().canonicalize().unwrap_or_default()))
        .ok_or_else(|| eyre::eyre!("No disk found for path"))?;

    let usage = StatsDiskUsage {
        fstype: disk.file_system().to_string_lossy().into_owned(),
        size: disk.total_space(),
        used: disk.total_space() - disk.available_space(),
        free: disk.available_space(),
    };
    Ok(usage)
}

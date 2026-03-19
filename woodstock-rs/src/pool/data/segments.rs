use std::path::{Path, PathBuf};

use eyre::{Result, WrapErr};
use tokio::fs::{copy, create_dir_all, remove_file, rename};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::utils::lock_redis::PoolLockRedis;

use super::{
    create_segment, resolve_segment_path, segment_path, segments_directory_path,
    IndexedSegment, PoolIndex, SegmentFile,
};

pub struct SegmentReservation {
    segment: IndexedSegment,
    segment_path: PathBuf,
    lock: PoolLockRedis,
}

impl SegmentReservation {
    #[must_use]
    pub fn segment(&self) -> &IndexedSegment {
        &self.segment
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.segment_path
    }

    #[must_use]
    pub fn cancellation_token(&self) -> &CancellationToken {
        self.lock.cancellation_token()
    }

    #[must_use]
    pub fn into_parts(self) -> (IndexedSegment, PathBuf, PoolLockRedis) {
        (self.segment, self.segment_path, self.lock)
    }
}

pub struct CompactionTarget {
    segment: IndexedSegment,
    segment_file: SegmentFile,
    temp_path: PathBuf,
    final_path: PathBuf,
    _lock: PoolLockRedis,
}

impl CompactionTarget {
    #[must_use]
    pub fn segment(&self) -> &IndexedSegment {
        &self.segment
    }

    #[must_use]
    pub fn segment_mut(&mut self) -> &mut IndexedSegment {
        &mut self.segment
    }

    #[must_use]
    pub fn segment_file(&self) -> &SegmentFile {
        &self.segment_file
    }

    #[must_use]
    pub fn segment_file_mut(&mut self) -> &mut SegmentFile {
        &mut self.segment_file
    }

    #[must_use]
    pub fn temp_path(&self) -> &Path {
        &self.temp_path
    }

    #[must_use]
    pub fn final_path(&self) -> &Path {
        &self.final_path
    }

    pub async fn publish(&mut self) -> Result<()> {
        if self.temp_path != self.final_path {
            if let Some(parent) = self.final_path.parent() {
                create_dir_all(parent).await?;
            }

            rename(&self.temp_path, &self.final_path)
                .await
                .wrap_err_with(|| {
                    format!(
                        "failed to publish compaction target {}",
                        self.final_path.display()
                    )
                })?;
        }

            self.segment_file.relocate(self.final_path.clone());
            self.segment.state = self.segment_file.state();
            self.segment.size_total = self.segment_file.size_total();
        self.temp_path = self.final_path.clone();
        Ok(())
    }
}

#[must_use]
pub fn segment_write_lock_resource(segment_id: u64) -> String {
    format!("pool:segment:write:{segment_id}")
}

#[must_use]
pub fn segment_compaction_lock_resource(segment_id: u64) -> String {
    format!("pool:segment:compact:{segment_id}")
}

#[must_use]
pub fn compaction_temp_segment_path(pool_path: &Path, segment_id: u64) -> PathBuf {
    segments_directory_path(pool_path).join(format!(
        ".seg-{segment_id:011}.{}.compacting",
        Uuid::new_v4()
    ))
}

pub async fn reserve_open_or_create_segment(
    pool_path: &Path,
    redis_url: &str,
    segment_index: &PoolIndex,
    target_size: u64,
) -> Result<SegmentReservation> {
    for segment in segment_index.list_open_segments()? {
        if let Some(reservation) =
            try_reserve_existing_segment(pool_path, redis_url, segment).await?
        {
            return Ok(reservation);
        }
    }

    create_and_reserve_segment(pool_path, redis_url, segment_index, target_size).await
}

pub async fn try_reserve_existing_segment(
    pool_path: &Path,
    redis_url: &str,
    segment: IndexedSegment,
) -> Result<Option<SegmentReservation>> {
    let Some(lock) = try_reserve_write_lock(redis_url, segment.segment_id).await? else {
        return Ok(None);
    };

    Ok(Some(SegmentReservation {
        segment_path: resolve_segment_path(pool_path, &segment),
        segment,
        lock,
    }))
}

pub async fn create_and_reserve_segment(
    pool_path: &Path,
    redis_url: &str,
    segment_index: &PoolIndex,
    target_size: u64,
) -> Result<SegmentReservation> {
    let segment_id = segment_index.allocate_next_segment_id()?;
    let lock = try_reserve_write_lock(redis_url, segment_id)
        .await?
        .ok_or_else(|| eyre::eyre!("new segment {segment_id} was unexpectedly locked"))?;
    let created_segment = create_segment(pool_path, segment_id, target_size).await?;
    let segment_path = created_segment.path;
    let segment = created_segment.segment;
    segment_index.add_segment(&segment)?;

    Ok(SegmentReservation {
        segment,
        segment_path,
        lock,
    })
}

pub async fn try_reserve_segment_compaction_lock(
    redis_url: &str,
    segment_id: u64,
) -> Result<Option<PoolLockRedis>> {
    PoolLockRedis::new(
        redis_url,
        &segment_compaction_lock_resource(segment_id),
        format!("segment_compaction:{segment_id}"),
    )
    .await?
    .try_lock_exclusive_nowait()
    .await
}

pub async fn create_compaction_target(
    pool_path: &Path,
    redis_url: &str,
    segment_index: &PoolIndex,
    target_size: u64,
) -> Result<CompactionTarget> {
    let segment_id = segment_index.allocate_next_segment_id()?;
    let lock = try_reserve_write_lock(redis_url, segment_id)
        .await?
        .ok_or_else(|| {
            eyre::eyre!("new compaction target segment {segment_id} was unexpectedly locked")
        })?;
    let temp_path = compaction_temp_segment_path(pool_path, segment_id);
    let final_path = segment_path(pool_path, segment_id);

    let segment_file = SegmentFile::create(&temp_path, segment_id, target_size).await?;
    let segment = IndexedSegment {
        segment_id,
        state: segment_file.state(),
        size_total: segment_file.size_total(),
        size_effective: 0,
        size_limit: target_size,
        chunk_count: 0,
    };

    Ok(CompactionTarget {
        segment,
        segment_file,
        temp_path,
        final_path,
        _lock: lock,
    })
}

pub async fn cleanup_unpublished_compaction_targets(targets: &[CompactionTarget]) {
    for target in targets {
        for path in [target.temp_path(), target.final_path()] {
            if !path.exists() {
                continue;
            }

            let _ = remove_file(path).await;
        }
    }
}

pub async fn cleanup_compacted_segment(source_path: &Path, target: Option<&Path>) -> Result<()> {
    if !source_path.exists() {
        return Ok(());
    }

    if let Some(target_directory) = target {
        create_dir_all(target_directory).await?;
        let destination = target_directory.join(source_path.file_name().unwrap_or_default());
        match rename(source_path, &destination).await {
            Ok(()) => Ok(()),
            Err(rename_error) => match copy(source_path, &destination).await {
                Ok(_) => {
                    remove_file(source_path).await?;
                    Ok(())
                }
                Err(copy_error) => Err(std::io::Error::new(
                    copy_error.kind(),
                    format!("rename failed: {rename_error}; copy fallback failed: {copy_error}"),
                )
                .into()),
            },
        }
    } else {
        remove_file(source_path).await?;
        Ok(())
    }
}

async fn try_reserve_write_lock(redis_url: &str, segment_id: u64) -> Result<Option<PoolLockRedis>> {
    PoolLockRedis::new(
        redis_url,
        &segment_write_lock_resource(segment_id),
        format!("segment_write:{segment_id}"),
    )
    .await?
    .try_lock_exclusive_nowait()
    .await
}

#[cfg(test)]
mod tests {
    use eyre::Result;
    use tempfile::tempdir;

    use super::{
        compaction_temp_segment_path, segment_compaction_lock_resource, segment_write_lock_resource,
    };

    #[test]
    fn lock_resources_match_v3_convention() {
        assert_eq!(segment_write_lock_resource(44), "pool:segment:write:44");
        assert_eq!(
            segment_compaction_lock_resource(44),
            "pool:segment:compact:44"
        );
    }

    #[test]
    fn compaction_temp_segment_path_is_hidden_from_normal_layout() -> Result<()> {
        let directory = tempdir()?;
        let temp_path = compaction_temp_segment_path(directory.path(), 42);

        assert!(temp_path.to_string_lossy().contains(".seg-00000000042."));
        assert!(temp_path.to_string_lossy().contains(".compacting"));
        Ok(())
    }
}

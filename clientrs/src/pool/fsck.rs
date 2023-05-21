use async_walkdir::{Filtering, WalkDir};
use futures::{pin_mut, StreamExt};
use log::{error, info};

use crate::{
    config::{Backups, Configuration},
    pool::PoolChunkWrapper,
    PoolUnused,
};

use super::Refcnt;

pub struct FsckCount {
    pub error_count: usize,
    pub total_count: usize,
}

pub struct FsckUnusedCount {
    pub in_unused: usize,
    pub in_refcnt: usize,
    pub in_nothing: usize,
    pub missing: usize,
}

fn check_integrity(original_refcnt: &Refcnt, new_refcnt: &Refcnt) -> usize {
    let mut error_count = 0;

    for refcnt in new_refcnt.list_refcnt() {
        if let Some(original_refcnt) = original_refcnt.get_refcnt_copy(&refcnt.sha256) {
            if original_refcnt.ref_count != refcnt.ref_count {
                error_count += 1;
                error!(
                    "Refcnt mismatch {}: {} instead of {}",
                    hex::encode(&refcnt.sha256),
                    original_refcnt.ref_count,
                    refcnt.ref_count
                );
            }
        } else {
            error_count += 1;
            error!(
                "Refcnt mismatch {}: {} instead of {}",
                hex::encode(&refcnt.sha256),
                0,
                refcnt.ref_count
            );
        }
    }

    error_count
}

pub async fn check_backup_integrity(
    hostname: &str,
    backup_number: usize,
    dry_run: bool,
) -> Result<FsckCount, Box<dyn std::error::Error>> {
    let configuration = Configuration::default();
    let backups = Backups::new(&configuration.path);
    let destination_backup = backups.get_backup_destination_directory(hostname, backup_number);

    let new_refcnt = Refcnt::apply_from_backup(hostname, backup_number).await?;
    let mut original_refcnt = Refcnt::new(&destination_backup);
    original_refcnt.load_refcnt(false).await;

    let error_count = check_integrity(&original_refcnt, &new_refcnt);

    let mut new_refcnt = new_refcnt;

    if !dry_run && error_count > 0 {
        info!("Fix refcnt for {hostname}/{backup_number}");
        new_refcnt.finish().await?;
        new_refcnt.save_refcnt().await?;
    }

    Ok(FsckCount {
        error_count,
        total_count: new_refcnt.size(),
    })
}

pub async fn check_host_integrity(
    hostname: &str,
    dry_run: bool,
) -> Result<FsckCount, Box<dyn std::error::Error>> {
    let configuration = Configuration::default();
    let backups = Backups::new(&configuration.path);
    let destination_directory = backups.get_host_path(hostname);

    let new_refcnt = Refcnt::apply_from_host(hostname).await?;

    let mut original_refcnt = Refcnt::new(&destination_directory);
    original_refcnt.load_refcnt(false).await;

    let error_count = check_integrity(&original_refcnt, &new_refcnt);

    let mut new_refcnt = new_refcnt;

    if !dry_run && error_count > 0 {
        info!("Fix refcnt for {hostname}");
        new_refcnt.finish().await?;
        new_refcnt.save_refcnt().await?;
    }

    Ok(FsckCount {
        error_count,
        total_count: new_refcnt.size(),
    })
}

pub async fn check_pool_integrity(dry_run: bool) -> Result<FsckCount, Box<dyn std::error::Error>> {
    let configuration = Configuration::default();
    let mut pool_refcnt = Refcnt::new(&configuration.path.pool_path);
    pool_refcnt.load_refcnt(false).await;

    let new_refcnt = Refcnt::apply_from_all().await?;

    let error_count = check_integrity(&pool_refcnt, &new_refcnt);

    let mut new_refcnt = new_refcnt;

    if !dry_run && error_count > 0 {
        info!("Fix refcnt for pool");
        new_refcnt.finish().await?;
        new_refcnt.save_refcnt().await?;
    }

    Ok(FsckCount {
        error_count,
        total_count: new_refcnt.size(),
    })
}

pub async fn check_unused(
    dry_run: bool,
    cb: &impl Fn(usize),
) -> Result<FsckUnusedCount, Box<dyn std::error::Error>> {
    let configuration = Configuration::default();
    let mut pool_refcnt = Refcnt::new(&configuration.path.pool_path);
    pool_refcnt.load_refcnt(false).await;
    pool_refcnt.load_unused().await;

    let mut in_unused = 0;
    let mut in_refcnt = 0;
    let mut in_nothing = 0;
    let mut missing = 0;

    // FIXME: Remove walkdir and use unfold like get_files_recursive
    let entries = WalkDir::new(&configuration.path.pool_path)
        .filter(|f| async move {
            let path = f.path();

            if path.extension().is_some_and(|ext| ext == "zz") {
                Filtering::Continue
            } else {
                Filtering::Ignore
            }
        })
        .filter_map(|path| async move {
            if let Ok(path) = path {
                // Remove -sha256.zz from filename
                let filename = path.file_name().into_string().ok().unwrap_or_default();
                return hex::decode(&filename[..64]).ok();
            }
            None
        });
    pin_mut!(entries);

    while let Some(hash) = entries.next().await {
        let wrapper = PoolChunkWrapper::new(&configuration.path.pool_path, Some(&hash));
        let hash_str = wrapper.get_hash_str().as_ref().unwrap();

        if pool_refcnt.get_unused(&hash).is_some() {
            in_unused += 1;
            if pool_refcnt.get_refcnt(&hash).is_some() {
                in_refcnt += 1;
                cb(1);

                pool_refcnt.remove_unused(&hash);

                error!("{} is in unused and in refcnt", hash_str);
            }
        } else if pool_refcnt.get_refcnt(&hash).is_some() {
            in_refcnt += 1;
            cb(1);
        } else {
            let information = wrapper.chunk_information().await?;

            in_nothing += 1;
            pool_refcnt.add_unused(PoolUnused {
                sha256: hash.clone(),
                size: information.size,
                compressed_size: information.compressed_size,
            });
            error!("{} is not in unused nor in refcnt", hash_str);
        }
    }

    for refcnt in pool_refcnt.list_refcnt() {
        let wrapper = PoolChunkWrapper::new(&configuration.path.pool_path, Some(&refcnt.sha256));
        if !wrapper.exists() {
            missing += 1;
            error!("{} is missing", hex::encode(&refcnt.sha256));
        }
    }

    if !dry_run {
        pool_refcnt.save_unused().await?;
    }

    Ok(FsckUnusedCount {
        in_unused,
        in_refcnt,
        in_nothing,
        missing,
    })
}

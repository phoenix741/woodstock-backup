mod fsck;
mod pool_chunk_information;
mod pool_chunk_wrapper;
mod pool_chunk_wrapper_writer;
mod pool_refcnt;
mod refcnt;
mod utils;

pub use fsck::*;
pub use pool_chunk_information::*;
pub use pool_chunk_wrapper::*;
pub use pool_chunk_wrapper_writer::*;
pub use refcnt::*;
pub use utils::*;

use crate::{
    config::{Backups, Context},
    utils::lock::PoolLock,
};

use eyre::Result;
use log::{debug, info};
use std::time::SystemTime;

pub async fn add_refcnt_to_pool(
    ctxt: &Context,
    hostname: &str,
    backup_number: usize,
) -> Result<()> {
    let _lock = PoolLock::new(&ctxt.config.path.pool_path).lock().await?;

    let backups = Backups::new(ctxt);
    let from_directory = backups.get_backup_destination_directory(hostname, backup_number);

    let pool_directory = &ctxt.config.path.pool_path;

    info!("Add refcnt to pool for {}", from_directory.display());
    let mut backup_refcnt = Refcnt::new(&from_directory);
    backup_refcnt.load_refcnt(false).await;

    debug!("Save refcnt to pool");
    Refcnt::apply_all_from(
        pool_directory,
        &backup_refcnt,
        &RefcntApplySens::Increase,
        &SystemTime::now(),
        ctxt,
    )
    .await?;
    info!("Refcnt applied to pool");

    Ok(())
}

pub async fn remove_refcnt_to_pool(
    ctxt: &Context,
    hostname: &str,
    backup_number: usize,
) -> Result<()> {
    let _lock = PoolLock::new(&ctxt.config.path.pool_path).lock().await?;

    let backups = Backups::new(ctxt);
    let from_directory = backups.get_backup_destination_directory(hostname, backup_number);

    let pool_directory = &ctxt.config.path.pool_path;

    info!("Remove refcnt to pool for {}", from_directory.display());
    let mut backup_refcnt = Refcnt::new(&from_directory);
    backup_refcnt.load_refcnt(false).await;

    debug!("Remove refcnt to pool");
    Refcnt::apply_all_from(
        pool_directory,
        &backup_refcnt,
        &RefcntApplySens::Decrease,
        &SystemTime::now(),
        ctxt,
    )
    .await?;
    info!("Refcnt removed from pool");

    Ok(())
}

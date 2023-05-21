use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use console::Term;
use indicatif::{HumanBytes, HumanCount, ProgressBar, ProgressStyle};
use log::{error, info};

use crate::{
    config::{Backups, Configuration, Hosts},
    pool::{
        check_backup_integrity, check_host_integrity, check_pool_integrity, check_unused,
        PoolChunkWrapper, Refcnt,
    },
};

pub async fn add_refcnt_to_pool(
    configuration: &Configuration,
    hostname: &str,
    backup_number: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let backups = Backups::new(&configuration.path);
    let destination_directory = backups.get_backup_destination_directory(hostname, backup_number);

    info!("Add refcnt to pool for {}", destination_directory.display());
    let mut backup_refcnt = Refcnt::new(&destination_directory);
    backup_refcnt.load_refcnt(false).await;

    info!("Apply refcnt to pool");
    Refcnt::apply_all_from(
        &configuration.path.pool_path,
        &backup_refcnt,
        &crate::pool::RefcntApplySens::Increase,
    )
    .await?;

    info!("Refcnt applied to pool");

    Ok(())
}

pub async fn remove_refcnt_to_pool(
    configuration: &Configuration,
    hostname: &str,
    backup_number: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let backups = Backups::new(&configuration.path);
    let destination_directory = backups.get_backup_destination_directory(hostname, backup_number);

    let mut backup_refcnt = Refcnt::new(&destination_directory);
    backup_refcnt.load_refcnt(false).await;

    Refcnt::apply_all_from(
        &configuration.path.pool_path,
        &backup_refcnt,
        &crate::pool::RefcntApplySens::Decrease,
    )
    .await?;

    Ok(())
}

pub async fn check_compression(
    configuration: &Configuration,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut pool_refcnt = Refcnt::new(&configuration.path.pool_path);
    pool_refcnt.load_refcnt(false).await;

    let mut compressed_size: u64 = 0;
    let mut uncompressed_size: u64 = 0;
    let mut error_count: u64 = 0;
    let mut total_count: u64 = 0;

    let term = Term::stdout();

    for refcnt in pool_refcnt.list_refcnt() {
        total_count += 1;
        compressed_size += refcnt.compressed_size;
        uncompressed_size += refcnt.size;

        if refcnt.compressed_size > refcnt.size {
            error_count += 1;
            let hash_str = hex::encode(&refcnt.sha256);
            error!(
                "{}: compressed size {} is greater than uncompressed size {}",
                hash_str,
                HumanBytes(refcnt.compressed_size),
                HumanBytes(refcnt.size)
            );
        }
    }

    term.write_line(&std::format!(
        "Total errors: {}/{}",
        HumanCount(error_count),
        HumanCount(total_count)
    ))?;
    term.write_line(&std::format!(
        "Total compressed size: {}",
        HumanBytes(compressed_size)
    ))?;
    term.write_line(&std::format!(
        "Total uncompressed size: {}",
        HumanBytes(uncompressed_size)
    ))?;

    Ok(())
}

pub async fn verify_chunk(configuration: &Configuration) -> Result<(), Box<dyn std::error::Error>> {
    let mut pool_refcnt = Refcnt::new(&configuration.path.pool_path);
    pool_refcnt.load_refcnt(false).await;
    pool_refcnt.load_unused().await;

    let mut chunks = pool_refcnt
        .list_refcnt()
        .map(|refcnt| refcnt.sha256.clone())
        .collect::<Vec<_>>();
    chunks.extend(
        pool_refcnt
            .list_unused()
            .map(|unused| unused.sha256.clone()),
    );

    let mut error_count: u64 = 0;
    let mut total_count: u64 = 0;

    let term = Term::stdout();
    let bar = ProgressBar::new(chunks.len() as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {percent_precise}% {human_pos}/{human_len} ETA: {eta}",
        )
        .unwrap(),
    );

    for refcnt in chunks {
        let wrapper = PoolChunkWrapper::new(&configuration.path.pool_path, Some(&refcnt));

        let is_valid = wrapper.check_chunk_information().await?;
        if !is_valid {
            error_count += 1;
        }

        total_count += 1;
        bar.inc(1);
    }

    bar.finish();

    term.write_line(&std::format!(
        "Total errors: {}/{}",
        HumanCount(error_count),
        HumanCount(total_count)
    ))?;

    Ok(())
}

pub async fn verify_refcnt(
    configuration: &Configuration,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let hosts = Hosts::new(&configuration.path);
    let backups = Backups::new(&configuration.path);

    let mut error_count = 0;
    let mut total_count = 0;
    let mut count = 1;

    for host in hosts.list_hosts()? {
        let backups = backups.get_backups(&host);
        count += backups.len() + 1;
    }

    let term = Term::stdout();
    let bar = ProgressBar::new(count as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {percent_precise}% {human_pos}/{human_len} ETA: {eta}",
        )
        .unwrap(),
    );

    for host in hosts.list_hosts()? {
        let backups = backups.get_backups(&host);
        for backup in backups {
            let result = check_backup_integrity(&host, backup.number, dry_run).await?;

            error_count += result.error_count;
            total_count += result.total_count;

            bar.inc(1);
        }

        let result = check_host_integrity(&host, dry_run).await?;

        error_count += result.error_count;
        total_count += result.total_count;

        bar.inc(1);
    }

    let result = check_pool_integrity(dry_run).await?;

    error_count += result.error_count;
    total_count += result.total_count;

    bar.inc(1);

    bar.finish();

    term.write_line(&std::format!(
        "Total errors: {}/{}",
        HumanCount(error_count as u64),
        HumanCount(total_count as u64)
    ))?;

    Ok(())
}

pub async fn verify_unused(
    configuration: &Configuration,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut pool_refcnt = Refcnt::new(&configuration.path.pool_path);
    pool_refcnt.load_refcnt(false).await;
    pool_refcnt.load_unused().await;

    let total = pool_refcnt.list_unused().count() + pool_refcnt.list_refcnt().count();

    let term = Term::stdout();
    let bar = ProgressBar::new(total as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {percent_precise}% {human_pos}/{human_len} ETA: {eta}",
        )
        .unwrap(),
    );

    let result = check_unused(dry_run, &|p| bar.inc(p as u64)).await?;

    bar.finish();

    term.write_line(&std::format!(
        "In Refcnt: {}",
        HumanCount(result.in_refcnt as u64)
    ))?;
    term.write_line(&std::format!(
        "In Unused: {}",
        HumanCount(result.in_unused as u64)
    ))?;
    term.write_line(&std::format!(
        "In Nothing: {}",
        HumanCount(result.in_nothing as u64)
    ))?;
    term.write_line(&std::format!(
        "In Missing: {}",
        HumanCount(result.missing as u64)
    ))?;

    Ok(())
}

pub async fn clean_unused_pool(
    configuration: &Configuration,
    target: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let target = target.map(PathBuf::from);
    let mut refcnt = Refcnt::new(&configuration.path.pool_path);
    refcnt.load_unused().await;

    let total = Arc::new(Mutex::new(0));

    let term = Term::stdout();
    let bar = ProgressBar::new(refcnt.list_unused().count() as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {percent_precise}% {human_pos}/{human_len} ETA: {eta}",
        )
        .unwrap(),
    );

    refcnt
        .remove_unused_files(configuration, target, &|unused| {
            let compressed_size = unused
                .clone()
                .map(|f| f.compressed_size)
                .unwrap_or_default();

            let mut total = total.lock().unwrap();
            *total += compressed_size;

            bar.inc(1);
        })
        .await?;

    bar.finish();

    term.write_line(&std::format!(
        "Total removed: {}",
        HumanBytes(*total.lock().unwrap())
    ))?;

    Ok(())
}

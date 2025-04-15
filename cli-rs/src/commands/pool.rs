use std::path::PathBuf;

use console::Term;
use eyre::Result;
use indicatif::{HumanBytes, HumanCount, ProgressBar, ProgressStyle};
use log::error;

use woodstock::{
    config::Configuration, pool::Refcnt, server::pool_fsck::PoolFsck, utils::lock::PoolLock,
    EventSource,
};

pub async fn check_compression(config: &Configuration) -> Result<()> {
    let _lock = PoolLock::new(&config.path.pool_path).lock().await?;

    let mut pool_refcnt = Refcnt::new(&config.path.pool_path);
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

pub async fn verify_chunk(config: &Configuration, source: EventSource) -> Result<()> {
    let pool_fsck = PoolFsck::new(config);

    let max = pool_fsck.verify_chunk_max().await?;

    let term = Term::stdout();
    let bar = ProgressBar::new(max.len() as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {percent_precise}% {human_pos}/{human_len} ETA: {eta}",
        )
        .unwrap(),
    );

    let result = pool_fsck
        .verify_chunk(source, &|progress| {
            bar.set_position(progress.progress_current as u64);
        })
        .await?;

    bar.finish();

    term.write_line(&std::format!(
        "Total errors: {}/{}",
        HumanCount(result.error),
        HumanCount(result.count)
    ))?;

    Ok(())
}

pub async fn verify_refcnt(
    config: &Configuration,
    source: EventSource,
    dry_run: bool,
) -> Result<()> {
    let pool_fsck = PoolFsck::new(config);

    let total = pool_fsck.verify_refcnt_max().await?;

    let term = Term::stdout();
    let bar = ProgressBar::new(total as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {percent_precise}% {human_pos}/{human_len} ETA: {eta}",
        )
        .unwrap(),
    );

    let result = pool_fsck
        .verify_refcnt(dry_run, source, &|progress| {
            bar.set_position(progress.progress_current as u64);
        })
        .await?;

    bar.finish();

    term.write_line(&std::format!(
        "Total errors: {}/{}",
        HumanCount(result.error),
        HumanCount(result.count)
    ))?;

    Ok(())
}

pub async fn verify_unused(
    config: &Configuration,
    source: EventSource,
    dry_run: bool,
) -> Result<()> {
    let pool_fsck = PoolFsck::new(config);

    let total = pool_fsck.verify_unused_max().await?;

    let term = Term::stdout();
    let bar = ProgressBar::new(total as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {percent_precise}% {human_pos}/{human_len} ETA: {eta}",
        )
        .unwrap(),
    );

    let result = pool_fsck
        .verify_unused(dry_run, source, &|p| {
            bar.set_position(p.progress_current as u64);
        })
        .await?;

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
    config: &Configuration,
    source: EventSource,
    target: Option<String>,
) -> Result<()> {
    let pool_fsck = PoolFsck::new(config);

    let target = target.map(PathBuf::from);

    let total = pool_fsck.clean_unused_max().await?;

    let term = Term::stdout();
    let bar = ProgressBar::new(total as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {percent_precise}% {human_pos}/{human_len} ETA: {eta}",
        )
        .unwrap(),
    );

    let result = pool_fsck
        .clean_unused_pool(target, source, &|p| {
            bar.set_position(p.progress_current as u64);
        })
        .await?;

    bar.finish();

    term.write_line(&std::format!("Total removed: {}", HumanBytes(result.size)))?;

    Ok(())
}

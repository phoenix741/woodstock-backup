use eyre::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

use woodstock::{
    config::{Configuration, ConfigurationPath},
    server::pool_convert::PoolConvert,
    ChunkAlgorithm, EventSource,
};

pub async fn convert_hash_repo(backup_path: &str, hash: &str) -> Result<()> {
    let hash = ChunkAlgorithm::from_str_name(hash)
        .ok_or_else(|| eyre::eyre!("Invalid chunk algorithm: {}", hash))?;

    let backup_path = PathBuf::from(backup_path);
    let configuration = Configuration::from_backup_path(backup_path);

    let hosts_path = configuration
        .path
        .hosts_path
        .clone()
        .with_extension(hash.as_str_name());

    let pool_path = configuration
        .path
        .pool_path
        .clone()
        .with_extension(hash.as_str_name());

    let new_configuration = configuration.clone();
    let new_configuration = Configuration {
        path: ConfigurationPath {
            hosts_path,
            pool_path,
            ..new_configuration.path
        },
        chunk_algorithm: hash,
        ..new_configuration
    };

    // For each chunk in the pool path, calculate the new hash
    let pool_converter = PoolConvert::new(&configuration);

    let max = pool_converter.get_max().await?;

    let bar = ProgressBar::new(max as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {percent_precise}% {human_pos}/{human_len} ETA: {eta}",
        )
        .unwrap(),
    );

    let () = pool_converter
        .convert_backup_dir(&new_configuration, EventSource::Cli, &|progress| {
            bar.set_position(progress.progress_current as u64);
            bar.tick();
        })
        .await?;

    bar.finish();

    Ok(())
}

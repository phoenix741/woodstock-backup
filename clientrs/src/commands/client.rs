use std::path::Path;

use futures::{pin_mut, StreamExt};

use crate::{
    config::{Configuration, Hosts},
    manifest::PathManifest,
    scanner::{get_files, CreateManifestOptions},
    utils::path::{list_to_globset, vec_to_str},
};

pub async fn list_client_files(
    configuration: &Configuration,
    hostname: &str,
    share_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Start by reading the configuration file
    let hosts = Hosts::new(&configuration.path);
    let host = hosts.get_host(hostname)?;

    if let Some(operation) = host.operations.operation {
        let global_includes = operation.includes.unwrap_or_default();
        let global_excludes = operation.excludes.unwrap_or_default();

        for share in operation.shares {
            if share.name == share_path {
                let mut includes = share.includes.unwrap_or_default();
                let mut excludes = share.excludes.unwrap_or_default();

                includes.extend(global_includes.clone());
                excludes.extend(global_excludes.clone());

                let includes = vec_to_str(&includes);
                let includes = list_to_globset(&includes)
                    .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;
                let excludes = vec_to_str(&excludes);
                let excludes = list_to_globset(&excludes)
                    .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;

                let share_path = Path::new(&share_path);

                let mut backup_size: u64 = 0;

                let files = get_files(
                    share_path,
                    &includes,
                    &excludes,
                    &CreateManifestOptions {
                        with_acl: false,
                        with_xattr: false,
                    },
                );
                pin_mut!(files);
                while let Some(file) = files.next().await {
                    backup_size += file.size();

                    let file = file.path();
                    let path = file.to_string_lossy();
                    println!("{}", path);
                }

                println!("Total size: {}", backup_size);
            }
        }
    }

    Ok(())
}

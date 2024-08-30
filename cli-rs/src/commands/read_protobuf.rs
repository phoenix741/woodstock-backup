use std::{error::Error, ffi::OsString};

use clap::ValueEnum;
use futures::StreamExt;

use woodstock::{
    config::{Backups, Context},
    proto::ProtobufReader,
    FileManifest, FileManifestJournalEntry, PoolRefCount, PoolUnused,
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ProtobufFormat {
    FileManifest,
    FileManifestJournalEntry,
    RefCount,
    Unused,
}

pub async fn read_protobuf(
    path: &str,
    format: &ProtobufFormat,
    filter_name: &Option<String>,
    filter_chunks: &Option<String>,
) -> Result<(), Box<dyn Error>> {
    match format {
        ProtobufFormat::FileManifest => {
            let mut messages = ProtobufReader::<FileManifest>::new(path, true).await?;
            let mut messages = messages.into_stream();

            while let Some(message) = messages.next().await {
                let message = message?;

                // Filter the output by file name
                if let Some(filter_name) = filter_name {
                    let filter_name: OsString = filter_name.into();

                    let path = message.path();
                    let path = path.file_name();
                    if let Some(path) = path {
                        if path != filter_name {
                            continue;
                        }
                    }
                }

                // Filter the output by chunks
                if let Some(filter_chunks) = filter_chunks {
                    let filter_chunks = hex::decode(filter_chunks)?;
                    // Check if on of the chunks of the file is in the message
                    if !message.chunks.iter().any(|chunk| chunk == &filter_chunks) {
                        continue;
                    }
                }

                let message_str = match message.to_yaml() {
                    Ok(message_str) => message_str,
                    Err(err) => format!("Error: {}", err),
                };

                print!("{}", message_str);
            }
        }
        ProtobufFormat::FileManifestJournalEntry => {
            let mut messages = ProtobufReader::<FileManifestJournalEntry>::new(path, true).await?;
            let mut messages = messages.into_stream();

            while let Some(message) = messages.next().await {
                let message = message?;
                // Filter the output by file name
                if let Some(filter_name) = filter_name {
                    let filter_name: OsString = filter_name.into();

                    if let Some(manifest) = &message.manifest {
                        let path = manifest.path();
                        let path = path.file_name();
                        if let Some(path) = path {
                            if path != filter_name {
                                continue;
                            }
                        }
                    }
                }

                // Filter the output by chunks
                if let Some(filter_chunks) = filter_chunks {
                    let filter_chunks = hex::decode(filter_chunks)?;
                    // Check if on of the chunks of the file is in the message
                    if let Some(manifest) = &message.manifest {
                        if !manifest.chunks.iter().any(|chunk| chunk == &filter_chunks) {
                            continue;
                        }
                    }
                }

                let message_str = match message.to_yaml() {
                    Ok(message_str) => message_str,
                    Err(err) => format!("Error: {}", err),
                };

                print!("{}", message_str);
            }
        }
        ProtobufFormat::RefCount => {
            let mut messages = ProtobufReader::<PoolRefCount>::new(path, true).await?;
            let mut messages = messages.into_stream();

            while let Some(message) = messages.next().await {
                let message = message?;
                // Filter the output by chunks
                if let Some(filter_chunks) = filter_chunks {
                    let filter_chunks = hex::decode(filter_chunks)?;

                    if message.sha256 != filter_chunks {
                        continue;
                    }
                }

                print!("{message}");
            }
        }
        ProtobufFormat::Unused => {
            let mut messages = ProtobufReader::<PoolUnused>::new(path, true).await?;
            let mut messages = messages.into_stream();

            while let Some(message) = messages.next().await {
                let message = message?;
                // Filter the output by chunks
                if let Some(filter_chunks) = filter_chunks {
                    let filter_chunks = hex::decode(filter_chunks)?;

                    if message.sha256 != filter_chunks {
                        continue;
                    }
                }

                print!("{message}");
            }
        }
    };

    Ok(())
}

pub async fn read_log(
    ctxt: &Context,
    hostname: &str,
    backup_number: usize,
    share_path: &str,
) -> Result<(), Box<dyn Error>> {
    let backups_services = Backups::new(ctxt);
    let manifest = backups_services.get_manifest(hostname, backup_number, share_path);
    let log_path = manifest.log_path;

    let mut messages = ProtobufReader::<FileManifestJournalEntry>::new(log_path, true).await?;
    let mut messages = messages.into_stream();

    while let Some(message) = messages.next().await {
        let message = message?;
        let log_line = message.to_log();

        println!("{log_line}");
    }

    Ok(())
}

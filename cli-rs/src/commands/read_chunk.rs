use console::Term;
use flate2::bufread::ZlibDecoder;
use log::info;
use std::{
    error::Error,
    fs::File,
    io::{stdout, BufReader, Read, Write},
    path::Path,
};

use futures::{pin_mut, StreamExt};
use woodstock::{
    config::{Backups, Context, Hosts, BUFFER_SIZE},
    pool::PoolChunkWrapper,
};

pub fn read_chunk(pool_path: &Path, chunk: &str) -> Result<(), Box<dyn Error>> {
    let chunk = hex::decode(chunk)?;
    let chunk = PoolChunkWrapper::new(pool_path, Some(&chunk));

    if !chunk.exists() {
        return Err("Chunk doesn't exist".into());
    }

    let chunk_path = chunk.chunk_path();

    // Read all the chunk content
    let file = File::open(chunk_path)?;
    let file = BufReader::new(file);
    let mut file = ZlibDecoder::new(file);

    let mut buffer = vec![0; BUFFER_SIZE];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }

        stdout().write_all(&buffer[..read]).unwrap();
    }

    Ok(())
}

pub async fn search_chunk(ctxt: &Context, chunk: &str) -> Result<(), Box<dyn Error>> {
    let term = Term::stdout();

    let chunk = hex::decode(chunk)?;

    let hosts_config = Hosts::new(ctxt);
    let backups_config = Backups::new(ctxt);

    let hosts = hosts_config.list_hosts().await.unwrap_or_default();
    for host in hosts {
        let backups = backups_config.get_backups(&host).await;
        for backup in backups {
            let manifests = backups_config.get_manifests(&host, backup.number).await;

            for manifest in manifests {
                info!(
                    "Process {}/{}/{}",
                    host, backup.number, manifest.manifest_name
                );
                let entries = manifest.read_all_message();
                pin_mut!(entries);

                while let Some(entry) = entries.next().await {
                    let found = entry.chunks.iter().filter(|c| chunk.eq(*c)).count();
                    if found > 0 {
                        term.write_line(&std::format!(
                            "{} Chunk found in backup of host {}/{}/{:?}",
                            found,
                            host,
                            backup.number,
                            entry.path()
                        ))?;
                    }
                }
            }
        }
    }

    Ok(())
}

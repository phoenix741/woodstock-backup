use futures::stream;
use log::debug;
use log::error;
use std::cmp::max;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

use futures::pin_mut;
use futures::StreamExt;

use crate::config::Backups;
use crate::config::Context;
use crate::config::Hosts;
use crate::proto::save_file;
use crate::statistics::write_statistics;
use crate::statistics::PoolStatistics;
use crate::PoolUnused;
use crate::{proto::ProtobufReader, PoolRefCount};

use super::PoolChunkWrapper;

pub enum RefcntApplySens {
    Increase,
    Decrease,
}

pub struct Refcnt<'context> {
    context: &'context Context,
    path: PathBuf,
    refcnt_path: PathBuf,
    unused_path: PathBuf,
    index: HashMap<Vec<u8>, PoolRefCount>,
    unused: HashMap<Vec<u8>, PoolUnused>,
    statistics: PoolStatistics,
}

// FIXME: Add lock (or add it on nodejs part ?)
impl<'context> Refcnt<'context> {
    #[must_use]
    pub fn new(path: &Path, ctxt: &'context Context) -> Self {
        Self {
            context: ctxt,
            path: path.to_path_buf(),
            refcnt_path: path.join("REFCNT"),
            unused_path: path.join("unused"),
            index: HashMap::new(),
            unused: HashMap::new(),
            statistics: PoolStatistics::default(),
        }
    }

    pub fn list_refcnt(&self) -> impl Iterator<Item = &PoolRefCount> {
        self.index.values()
    }

    pub fn list_unused(&self) -> impl Iterator<Item = &PoolUnused> {
        self.unused.values()
    }

    #[must_use]
    pub fn size(&self) -> usize {
        self.index.len()
    }

    #[must_use]
    pub fn get_unused(&self, key: &Vec<u8>) -> Option<&PoolUnused> {
        self.unused.get(key)
    }

    #[must_use]
    pub fn get_refcnt(&self, key: &Vec<u8>) -> Option<&PoolRefCount> {
        self.index.get(key)
    }

    pub fn add_unused(&mut self, unused: PoolUnused) {
        self.unused.insert(unused.sha256.clone(), unused);
    }

    pub fn remove_unused(&mut self, key: &Vec<u8>) {
        self.unused.remove(key);
    }

    #[must_use]
    pub fn get_refcnt_copy(&self, key: &Vec<u8>) -> Option<PoolRefCount> {
        match self.index.get(key).cloned() {
            Some(refcnt) => Some(refcnt),
            None => self.unused.get(key).map(|r| PoolRefCount {
                sha256: r.sha256.clone(),
                ref_count: 0,
                size: r.size,
                compressed_size: r.compressed_size,
            }),
        }
    }

    pub async fn apply_from_backup(
        hostname: &str,
        backup_number: usize,
        ctxt: &'context Context,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let backups = Backups::new(ctxt);
        let refcnt_file = backups.get_backup_destination_directory(hostname, backup_number);

        let manifests = backups.get_manifests(hostname, backup_number);
        let mut refcnt = Refcnt::new(&refcnt_file, ctxt);
        refcnt.load_refcnt(true).await;

        for manifest in manifests {
            let counts = manifest.generate_refcnt();
            pin_mut!(counts);

            while let Some(count) = counts.next().await {
                refcnt.apply(&count, &crate::pool::RefcntApplySens::Increase);
            }
        }

        Ok(refcnt)
    }

    pub async fn apply_from_host(
        hostname: &str,
        ctxt: &'context Context,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let backups_config = Backups::new(ctxt);
        let refcnt_file = backups_config.get_host_path(hostname);

        let backups = backups_config.get_backups(hostname);

        let mut new_refcnt = Refcnt::new(&refcnt_file, ctxt);

        for backup in backups {
            let destination_backup =
                backups_config.get_backup_destination_directory(hostname, backup.number);

            let mut backup_refcnt = Refcnt::new(&destination_backup, ctxt);
            backup_refcnt.load_refcnt(false).await;

            for pool_refcnt in backup_refcnt.index.values() {
                new_refcnt.apply(pool_refcnt, &RefcntApplySens::Increase);
            }
        }

        Ok(new_refcnt)
    }

    pub async fn apply_from_all(
        ctxt: &'context Context,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let hosts_config = Hosts::new(ctxt);
        let backups_config = Backups::new(ctxt);

        let refcnt_file = ctxt.config.path.pool_path.clone();

        let hosts = hosts_config.list_hosts()?;

        let mut new_refcnt = Refcnt::new(&refcnt_file, ctxt);

        for host in hosts {
            let destination_host = backups_config.get_host_path(&host);

            let mut host_refcnt = Refcnt::new(&destination_host, ctxt);
            host_refcnt.load_refcnt(false).await;

            for pool_refcnt in host_refcnt.index.values() {
                new_refcnt.apply(pool_refcnt, &RefcntApplySens::Increase);
            }
        }

        Ok(new_refcnt)
    }

    pub async fn apply_all_from<'a>(
        path: &Path,
        refcnt: &'a Refcnt<'a>,
        sens: &RefcntApplySens,
        ctxt: &'context Context,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut new_refcnt = Refcnt::new(path, ctxt);
        new_refcnt.load_refcnt(false).await;
        new_refcnt.load_unused().await;

        debug!("Apply refcnt from {:?}", refcnt.path);
        for pool_refcnt in refcnt.index.values() {
            new_refcnt.apply(pool_refcnt, sens);
        }

        new_refcnt.finish().await?;

        new_refcnt.save_refcnt().await?;
        new_refcnt.save_unused().await?;

        Ok(new_refcnt)
    }

    pub async fn load_refcnt(&mut self, fill_zero: bool) {
        debug!("Load refcnt from {:?}", self.refcnt_path);
        let messages = ProtobufReader::<PoolRefCount>::new(&self.refcnt_path, true).await;

        self.index.clear();
        if let Ok(mut messages) = messages {
            let messages = messages.into_stream();
            pin_mut!(messages);

            while let Some(refcnt) = messages.next().await {
                if let Ok(mut refcnt) = refcnt {
                    let sha256_pool_refcnt = &refcnt.sha256;
                    if fill_zero {
                        refcnt.ref_count = 0;
                    }
                    self.index.insert(sha256_pool_refcnt.clone(), refcnt);
                }
            }
        }
    }

    pub async fn load_unused(&mut self) {
        debug!("Load unused from {:?}", self.unused_path);
        let messages = ProtobufReader::<PoolUnused>::new(&self.unused_path, true).await;

        self.unused.clear();
        if let Ok(mut messages) = messages {
            let messages = messages.into_stream();
            pin_mut!(messages);

            while let Some(unused) = messages.next().await {
                if let Ok(unused) = unused {
                    let sha256_pool_unused = &unused.sha256;
                    self.unused.insert(sha256_pool_unused.clone(), unused);
                }
            }
        }
    }

    pub async fn remove_unused_files(
        &mut self,
        target: Option<PathBuf>,
        callback: &impl Fn(&Option<PoolUnused>),
    ) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Remove unused files");
        let unused = self.unused.values().cloned().collect::<Vec<PoolUnused>>();
        for pool_unused in unused {
            let wrapper = PoolChunkWrapper::new(
                &self.context.config.path.pool_path,
                Some(&pool_unused.sha256),
            );
            if let Some(target) = &target {
                if let Err(e) = wrapper.mv(target).await {
                    error!("Error while moving chunk: {:?}", e);
                }
            } else if let Err(e) = wrapper.remove().await {
                error!("Error while removing chunk: {:?}", e);
            }

            let unused = self.unused.remove(&pool_unused.sha256);

            callback(&unused);
        }

        self.save_unused().await?;

        Ok(())
    }

    pub fn apply(&mut self, refcnt: &PoolRefCount, sens: &RefcntApplySens) {
        let sha256_pool_refcnt = &refcnt.sha256;
        let hash_str = hex::encode(sha256_pool_refcnt);

        let cnt = self
            .index
            .get(sha256_pool_refcnt)
            .cloned()
            .unwrap_or_default();

        let ref_count = match sens {
            RefcntApplySens::Increase => cnt.ref_count + refcnt.ref_count,
            RefcntApplySens::Decrease => cnt.ref_count.saturating_sub(refcnt.ref_count),
        };

        if cnt.compressed_size != refcnt.compressed_size
            && cnt.compressed_size != 0
            && refcnt.compressed_size != 0
        {
            error!("Registered compressed size is different for {hash_str}");
        }

        if cnt.size != refcnt.size && cnt.size != 0 && refcnt.size != 0 {
            error!("Registered compressed size is different for {hash_str}");
        }

        self.index.insert(
            sha256_pool_refcnt.clone(),
            PoolRefCount {
                sha256: sha256_pool_refcnt.clone(),
                ref_count,
                size: if refcnt.size == 0 {
                    cnt.size
                } else {
                    refcnt.size
                },
                compressed_size: if refcnt.compressed_size == 0 {
                    cnt.compressed_size
                } else {
                    refcnt.compressed_size
                },
            },
        );
    }

    pub async fn finish(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Read chunk informations");
        // For each value check chunk informations
        for (sha256_pool_refcnt, pool_refcnt) in &mut self.index {
            if pool_refcnt.size == 0 || pool_refcnt.compressed_size == 0 {
                let wrapper = PoolChunkWrapper::new(
                    &self.context.config.path.pool_path,
                    Some(sha256_pool_refcnt),
                );
                let informations = wrapper.chunk_information().await?;

                pool_refcnt.size = informations.size;
                pool_refcnt.compressed_size = informations.compressed_size;
            }
        }

        debug!("Calculate statistics");
        // For each value in the index
        for pool_refcnt in self.index.values() {
            let sha256_pool_refcnt = &pool_refcnt.sha256;
            self.statistics.nb_ref += max(pool_refcnt.ref_count, 0);

            if pool_refcnt.ref_count > self.statistics.longest_chain {
                self.statistics.longest_chain = pool_refcnt.ref_count;
            }

            match pool_refcnt.ref_count.cmp(&0) {
                std::cmp::Ordering::Greater => {
                    if self.unused.contains_key(sha256_pool_refcnt) {
                        self.unused.remove(sha256_pool_refcnt);
                    }

                    self.statistics.size += pool_refcnt.size;
                    self.statistics.compressed_size += pool_refcnt.compressed_size;
                }
                std::cmp::Ordering::Equal | std::cmp::Ordering::Less => {
                    self.unused.insert(
                        sha256_pool_refcnt.clone(),
                        PoolUnused {
                            sha256: sha256_pool_refcnt.clone(),
                            size: pool_refcnt.size,
                            compressed_size: pool_refcnt.compressed_size,
                        },
                    );
                }
            }
        }

        self.statistics.nb_chunk = u32::try_from(self.index.len()).unwrap_or_default();

        self.statistics.unused_size = self
            .unused
            .values()
            .map(|pool_unused| pool_unused.size)
            .sum();

        Ok(())
    }

    pub async fn save_refcnt(&self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Save refcnt");
        // Save the refcnt
        let source = stream::iter(self.index.values().cloned()).filter_map(|refcnt| async {
            if refcnt.ref_count > 0 {
                Some(refcnt)
            } else {
                None
            }
        });
        save_file(&self.refcnt_path, source, true, true).await?;

        write_statistics(&self.statistics, &self.path, &SystemTime::now()).await?;

        Ok(())
    }

    pub async fn save_unused(&self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Save unused");
        // Save the unused
        let source = stream::iter(self.unused.values().cloned());
        save_file(&self.unused_path, source, true, true).await?;

        Ok(())
    }
}

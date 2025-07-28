use crate::{
    config::{Backup, Backups, Configuration, Hosts},
    utils::path::{osstr_to_vec, vec_to_path},
    FileManifestStat, FileManifestType,
};
use crate::{
    utils::path::{path_to_vec, unique},
    FileManifest,
};
use eyre::{eyre, Ok, Result};
use lru::LruCache;
use tracing::info;
use uuid::Uuid;

use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    sync::Arc,
};
use std::{collections::HashSet, num::NonZeroUsize};
use tokio::io::AsyncBufRead;

impl FileManifest {
    /// Creates a new `FileManifest` from a host name.
    ///
    /// # Arguments
    /// * `host` - The name of the host.
    ///
    /// # Returns
    ///
    /// A new `FileManifest` instance representing the host.
    #[must_use]
    pub fn from_host(host: &str) -> Self {
        Self {
            path: path_to_vec(PathBuf::from(host)),
            stats: Some(FileManifestStat {
                mode: 0o755,
                file_type: FileManifestType::Directory as i32,
                ..FileManifestStat::default()
            }),
            ..Default::default()
        }
    }

    /// Creates a new `FileManifest` from a backup.
    ///
    /// # Arguments
    /// * `backup` - The backup to create the manifest from.
    ///
    /// # Returns
    ///
    /// A new `FileManifest` instance representing the backup.
    #[must_use]
    pub fn from_backup(backup: &Backup) -> Self {
        Self {
            path: path_to_vec(PathBuf::from(backup.id.to_string())),
            stats: Some(FileManifestStat {
                mode: 0o755,
                file_type: FileManifestType::Directory as i32,
                size: backup.file_size,
                compressed_size: backup.compressed_file_size,
                created: backup.start_date.timestamp(),
                last_modified: backup
                    .end_date
                    .and_then(|f| Some(f.timestamp()))
                    .unwrap_or_default(),
                ..FileManifestStat::default()
            }),
            ..Default::default()
        }
    }

    /// Creates a new `FileManifest` from a share name.
    ///
    /// # Arguments
    /// * `share` - The name of the share.
    ///
    /// # Returns
    ///
    /// A new `FileManifest` instance representing the share.
    #[must_use]
    pub fn from_share(share: &str) -> Self {
        Self {
            path: path_to_vec(PathBuf::from(share)),
            stats: Some(FileManifestStat {
                mode: 0o755,
                file_type: FileManifestType::Directory as i32,
                ..FileManifestStat::default()
            }),
            ..Default::default()
        }
    }

    /// Creates a new `FileManifest` from a file path.
    ///
    /// # Arguments
    /// * `file` - The file path as an `OsStr`.
    ///
    /// # Returns
    ///
    /// A new `FileManifest` instance representing the file.
    #[must_use]
    pub fn from_file(file: &OsStr) -> Self {
        Self {
            path: path_to_vec(PathBuf::from(file)),
            stats: Some(FileManifestStat {
                mode: 0o755,
                file_type: FileManifestType::Directory as i32,
                ..FileManifestStat::default()
            }),
            ..Default::default()
        }
    }
}

/// Represents the result of comparing the start of two path components.
#[derive(Debug, PartialEq)]
enum ComponentsStartWith {
    /// The components start with the base path.
    Base,
    /// The components are equal.
    Equal,
    /// The components start with a different prefix.
    Other(OsString),
    /// The components differ completely.
    Diff,
}

fn components_start_with(
    base: &std::path::Components,
    other: &std::path::Components,
) -> ComponentsStartWith {
    let mut base = base.clone();
    let mut other = other.clone();

    loop {
        let base_component = base.next();
        let other_component = other.next();

        match (base_component, other_component) {
            (Some(base_component), Some(other_component)) => {
                if base_component != other_component {
                    return ComponentsStartWith::Diff;
                }
            }
            (None, None) => return ComponentsStartWith::Equal,
            (Some(_), None) => {
                return ComponentsStartWith::Base;
            }
            (None, Some(base)) => {
                return ComponentsStartWith::Other(base.as_os_str().to_os_string())
            }
        }
    }
}

#[derive(Debug)]
pub struct SelectedShares {
    pub shares: Vec<String>,
    pub selected_share: Option<String>,
}

#[derive(Debug, Default)]
pub struct PathInformation<'a> {
    pub hostname: Option<&'a str>,
    pub backup_id: Option<Uuid>,
    pub shares: Vec<String>,
    pub share: Option<String>,
    pub path: Option<PathBuf>,
}

///
/// The goal of this module is to provide a view for the Woodstock application.
///
/// The view can be used to access to the directory as files
///
/// TODO: Remplace LruCache with a redis cache
pub struct WoodstockView {
    /// The hosts configuration for the Woodstock application.
    hosts: Arc<Hosts>,
    /// The backups configuration for the Woodstock application.
    backups: Arc<Backups>,
    /// The path to the storage pool.
    pool_path: PathBuf,
    /// A cache for storing file manifests.
    cache: LruCache<PathBuf, Vec<FileManifest>>,
}

impl WoodstockView {
    /// Creates a new `WoodstockView` instance.
    ///
    /// # Arguments
    /// * `config` - The configuration for the Woodstock application.
    ///
    /// # Panics
    ///
    /// This function will panic if the cache size in the configuration is zero.
    /// Ensure that the cache size is set to a non-zero value before calling this function.
    #[must_use]
    pub fn new(config: Arc<Configuration>, hosts: Arc<Hosts>, backups: Arc<Backups>) -> Self {
        Self {
            hosts,
            backups,
            pool_path: config.path.pool_path.clone(),
            // SAFETY: cache_size is validated at config load time to be ≥ 1
            cache: LruCache::new(NonZeroUsize::new(config.cache_size.max(1)).unwrap()),
        }
    }

    /// Retrieves the manifest from the cache or loads it if not present.
    ///
    /// # Arguments
    /// * `hostname` - The hostname of the backup.
    /// * `backup_number` - The backup number.
    /// * `share` - The share name.
    ///
    /// # Returns
    ///
    /// * `Ok(&Vec<FileManifest>)` if the manifest is successfully retrieved.
    /// * `Err(eyre::Report)` if an error occurs during retrieval.
    async fn get_manifest_from_cache(
        &mut self,
        hostname: &str,
        backup_id: Uuid,
        share: &str,
    ) -> Result<&Vec<FileManifest>> {
        let key = PathBuf::from([hostname, backup_id.to_string().as_str(), share].join("/"));

        let cached_result = self.cache.get_or_insert_mut(key, Vec::new);
        if cached_result.is_empty() {
            let manifest = self.backups.get_manifest(hostname, backup_id, share);
            *cached_result = manifest.read_manifest_entries_to_end().await?;
        }

        Ok(cached_result)
    }

    /// Lists the shares of the specified path.
    ///
    /// # Arguments
    ///
    /// * `hostname` - The hostname of the backup.
    /// * `backup_number` - The backup number.
    /// * `path` - The path to the file.
    ///
    /// # Returns
    ///
    /// A tuple containing the shares, the selected share, and the share size.
    ///
    /// # Errors
    ///
    /// An error can't be returned if the hosts, backup, can't be read
    async fn list_shares_of(
        &self,
        hostname: &str,
        backup_id: Uuid,
        path: &mut std::path::Components<'_>,
    ) -> Result<SelectedShares> {
        let mut shares = self
            .backups
            .get_backup_share_paths(hostname, backup_id)
            .await;

        // Ensure that shares are sorted by length (longest last) to ensure that the selected share is the share that
        // is the most specific
        shares.sort_by_key(std::string::String::len);

        let mut selected_share: Option<String> = None;

        // Filter the shares that are not in the path
        let shares: Vec<_> = shares
            .into_iter()
            .filter_map(|share| {
                // Save the position of the path
                let relative_share = share.trim_start_matches('/');
                let relative_share = PathBuf::from(relative_share);
                let share_path = relative_share.components();

                match components_start_with(path, &share_path) {
                    ComponentsStartWith::Base | ComponentsStartWith::Equal => {
                        for _i in 0..share_path.count() {
                            path.next();
                        }
                        selected_share = Some(share);
                        None
                    }
                    ComponentsStartWith::Other(other) => {
                        Some(other.to_str().unwrap_or_default().to_string())
                    }
                    ComponentsStartWith::Diff => None,
                }
            })
            .collect();

        let shares = unique(shares);

        Ok(SelectedShares {
            shares,
            selected_share,
        })
    }

    /// Retrieves a file from the specified path.
    ///
    /// # Arguments
    /// * `hostname` - The name of the host.
    /// * `backup_number` - The backup number.
    /// * `path` - The path to the file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be found or if an I/O error occurs.
    async fn get_file_from_path(
        &mut self,
        hostname: &str,
        backup_id: Uuid,
        share: &str,
        path: &Path,
    ) -> Result<&FileManifest> {
        let entries = self
            .get_manifest_from_cache(hostname, backup_id, share)
            .await?;

        for entry in entries {
            let manifest_path = vec_to_path(&entry.path);

            if manifest_path == path {
                return Ok(entry);
            }
        }

        Err(eyre!("File not found"))
    }

    /// Lists all files in the specified directory.
    ///
    /// # Arguments
    /// * `hostname` - The name of the host.
    /// * `backup_number` - The backup number.
    /// * `share` - The share name.
    /// * `path` - The directory path.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be read or if an I/O error occurs.
    pub async fn list_all_files(
        &mut self,
        hostname: &str,
        backup_id: Uuid,
        share: &str,
        path: &PathBuf,
    ) -> Result<Vec<FileManifest>> {
        let entries = self
            .get_manifest_from_cache(hostname, backup_id, share)
            .await?;

        let mut files = Vec::new();
        for entry in entries {
            let manifest_path = vec_to_path(&entry.path);

            if manifest_path.starts_with(path) {
                files.push(entry.clone());
            }
        }

        Ok(files)
    }

    /// Lists files from a specific directory.
    ///
    /// # Arguments
    /// * `hostname` - The name of the host.
    /// * `backup_number` - The backup number.
    /// * `share` - The share name.
    /// * `path` - The directory path.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be read or if an I/O error occurs.
    pub async fn list_file_from_dir(
        &mut self,
        hostname: &str,
        backup_id: Uuid,
        share: &str,
        path: &PathBuf,
    ) -> Result<Vec<FileManifest>> {
        let entries = self
            .get_manifest_from_cache(hostname, backup_id, share)
            .await?;

        let mut missing_paths = HashSet::new();
        let mut files = HashMap::new();
        for entry in entries {
            let manifest_path = vec_to_path(&entry.path);

            if manifest_path.starts_with(path) {
                let rest_path = manifest_path.strip_prefix(path).unwrap_or(&manifest_path);
                let mut components = rest_path.components();
                let subpath: Option<std::path::Component<'_>> = components.next();

                if let Some(subpath) = subpath {
                    let subpath = subpath.as_os_str().to_owned();
                    if components.next().is_none() {
                        files.insert(
                            subpath.clone(),
                            FileManifest {
                                path: osstr_to_vec(&subpath),
                                ..entry.clone()
                            },
                        );
                    } else if !missing_paths.contains(&subpath) {
                        missing_paths.insert(subpath);
                    }
                }
            }
        }

        // Add unique missing paths
        for missing_path in missing_paths {
            if !files.contains_key(&missing_path) {
                files.insert(missing_path.clone(), FileManifest::from_file(&missing_path));
            }
        }

        Ok(files.into_values().collect())
    }

    /// Retrieves the backup information from a given path.
    ///
    /// # Arguments
    /// * `path` - The path to retrieve the backup information from.
    ///
    /// # Returns
    ///
    /// * `Ok(PathInformation)` if the backup information is successfully retrieved.
    /// * `Err(eyre::Report)` if an error occurs during retrieval.
    async fn backup_from_path<'a>(&self, path: &'a Path) -> Result<PathInformation<'a>> {
        // Remove first slash if at start
        let path = path.strip_prefix("/").unwrap_or(path);
        let mut path_components = path.components();

        let hostname = path_components.next();
        if hostname.is_none() {
            return Ok(PathInformation {
                hostname: None,
                backup_id: None,
                shares: vec![],
                share: None,
                path: None,
            });
        }
        // SAFETY: hostname is Some here — the is_none() check above already returns early
        let hostname = hostname.unwrap().as_os_str().to_str().unwrap_or_default();

        let backup_id_component = path_components.next();
        if backup_id_component.is_none() {
            return Ok(PathInformation {
                hostname: Some(hostname),
                backup_id: None,
                shares: vec![],
                share: None,
                path: None,
            });
        }
        // SAFETY: backup_id is Some here — the is_none() check above returns early
        let backup_str = backup_id_component
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap_or_default();
        let backup_id = if let core::result::Result::Ok(uuid) = Uuid::parse_str(backup_str) {
            uuid
        } else if let core::result::Result::Ok(n) = backup_str.parse::<usize>() {
            match self.backups.get_backup_by_number(hostname, n).await {
                Some(backup) => backup.id,
                None => {
                    return Ok(PathInformation {
                        hostname: Some(hostname),
                        backup_id: None,
                        shares: vec![],
                        share: None,
                        path: None,
                    })
                }
            }
        } else {
            return Ok(PathInformation {
                hostname: Some(hostname),
                backup_id: None,
                shares: vec![],
                share: None,
                path: None,
            });
        };

        let selected_shares = self
            .list_shares_of(hostname, backup_id, &mut path_components)
            .await?;

        match selected_shares.selected_share {
            None => Ok(PathInformation {
                hostname: Some(hostname),
                backup_id: Some(backup_id),
                shares: selected_shares.shares,
                share: None,
                path: None,
            }),
            Some(selected_share) => {
                let path_rest = path_components.as_path();

                Ok(PathInformation {
                    hostname: Some(hostname),
                    backup_id: Some(backup_id),
                    shares: selected_shares.shares,
                    share: Some(selected_share),
                    path: Some(path_rest.to_path_buf()),
                })
            }
        }
    }

    /// Lists files based on the specified path.
    ///
    /// # Arguments
    /// * `path` - The path to list files from.
    ///
    /// # Errors
    ///
    /// Returns an error if the path cannot be read or if an I/O error occurs.
    pub async fn list(&mut self, path: &Path) -> Result<Vec<FileManifest>> {
        info!("Listing path: {:?}", path);

        let path_info = self.backup_from_path(path).await?;

        match path_info {
            PathInformation {
                hostname: None,
                backup_id: None,
                shares: _,
                share: _,
                path: _,
            } => {
                let hosts = self.hosts.list_hosts().await?;
                Ok(hosts
                    .into_iter()
                    .map(|s| FileManifest::from_host(&s))
                    .collect())
            }
            PathInformation {
                hostname: Some(hostname),
                backup_id: None,
                shares: _,
                share: _,
                path: _,
            } => {
                let backups = self.backups.get_backups(hostname).await;
                Ok(backups
                    .into_iter()
                    .map(|s| FileManifest::from_backup(&s))
                    .collect())
            }
            PathInformation {
                hostname: Some(_),
                backup_id: Some(_),
                shares,
                share: None,
                path: _,
            } => Ok(shares
                .into_iter()
                .map(|s| FileManifest::from_share(&s))
                .collect()),
            PathInformation {
                hostname: Some(hostname),
                backup_id: Some(backup_id),
                shares,
                share: Some(share),
                path: path_rest,
            } => {
                let path_rest = path_rest.unwrap_or_default();

                let shares = shares
                    .into_iter()
                    .map(|s| FileManifest::from_share(&s))
                    .collect::<Vec<FileManifest>>();

                let files = self
                    .list_file_from_dir(hostname, backup_id, &share, &path_rest)
                    .await?;

                // Add detected shares to files
                let mut files = files
                    .into_iter()
                    .chain(shares)
                    .collect::<Vec<FileManifest>>();

                files.sort_by(|a, b| a.path.cmp(&b.path));

                Ok(files)
            }
            _ => Err(eyre!("Invalid path")),
        }
    }

    /// Retrieves attributes of a file based on the specified path.
    ///
    /// # Arguments
    /// * `path` - The path to the file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file attributes cannot be retrieved due to I/O issues.
    pub async fn get_attribute(&mut self, path: &Path) -> Result<&FileManifest> {
        let path_info = self.backup_from_path(path).await?;

        let Some(hostname) = path_info.hostname else {
            return Err(eyre!("Invalid path (missing hostname)"));
        };
        let Some(backup_id) = path_info.backup_id else {
            return Err(eyre!("Invalid path (missing backup number)"));
        };
        let Some(share) = path_info.share else {
            return Err(eyre!("Invalid path (missing share)"));
        };
        let Some(path) = path_info.path else {
            return Err(eyre!("Invalid path (missing path)"));
        };

        let manifest = self
            .get_file_from_path(hostname, backup_id, &share, &path)
            .await?;

        Ok(manifest)
    }

    /// Reads a file from the specified path.
    ///
    /// # Arguments
    /// * `path` - The path to the file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read due to I/O issues.
    pub async fn read_file(&mut self, path: &Path) -> Result<impl AsyncBufRead> {
        info!("Reading file: {:?}", path);

        let pool_path = self.pool_path.clone();
        let path_info = self.backup_from_path(path).await?;

        let Some(hostname) = path_info.hostname else {
            return Err(eyre!("Invalid path (missing hostname)"));
        };
        let Some(backup_id) = path_info.backup_id else {
            return Err(eyre!("Invalid path (missing backup number)"));
        };
        let Some(share) = path_info.share else {
            return Err(eyre!("Invalid path (missing share)"));
        };
        let Some(path) = path_info.path else {
            return Err(eyre!("Invalid path (missing path)"));
        };

        let manifest = self
            .get_file_from_path(hostname, backup_id, &share, &path)
            .await?;

        let reader = manifest.open_from_pool(&pool_path);

        Ok(reader)
    }
}

#[test]
fn test_components_start_with() {
    let base = Path::new("/").components();
    let share = Path::new("/home/").components();
    assert_eq!(
        components_start_with(&base, &share),
        ComponentsStartWith::Other("home".into())
    );

    let base = Path::new("/home/user/test").components();
    let share = Path::new("/home/user/test").components();
    assert_eq!(
        components_start_with(&base, &share),
        ComponentsStartWith::Equal
    );

    let base = Path::new("/home/user/test/other").components();
    let share = Path::new("/home/user/test").components();
    assert_eq!(
        components_start_with(&base, &share),
        ComponentsStartWith::Base
    );

    let base = Path::new("/home/user/test").components();
    let share = Path::new("/home/user").components();
    assert_eq!(
        components_start_with(&base, &share),
        ComponentsStartWith::Base
    );

    let base = Path::new("/home/user/test").components();
    let share = Path::new("/home/user/other").components();
    assert_eq!(
        components_start_with(&base, &share),
        ComponentsStartWith::Diff
    );
}

//! This module provides utilities for the virtual filesystem layer in Woodstock backups.
//!
//! It includes constants for time-to-live (TTL) values used in the FUSE filesystem implementation, controlling cache durations for hosts, backups, and other resources.
//!
//! # Errors
//!
//! This module itself does not return errors, but its constants are used in code that may fail if TTL values are misconfigured.
//!
//! # Panics
//!
//! This module does not explicitly panic.

use eyre::Result;
use log::{debug, info};
use lru::LruCache;
use std::hash::Hasher;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncBufRead, AsyncReadExt};
use tokio::sync::Mutex;
use twox_hash::XxHash64;
use woodstock::config::Configuration;
use woodstock::manifest::PathManifest;
use woodstock::utils::path::{osstr_to_vec, path_to_vec, vec_to_osstr};
use woodstock::view::WoodstockView;
use woodstock::{FileManifest, FileManifestType};

// extern crate fuser;
// extern crate libc;

use fuser::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEmpty, ReplyEntry,
    ReplyOpen, Request,
};
use libc::ENOENT;
use std::{collections::HashMap, ffi::OsStr};

/// Time-to-live (TTL) for host cache entries in the FUSE filesystem (24 hours).
const TTL_HOST: Duration = Duration::from_secs(86_400);
/// Time-to-live (TTL) for backup cache entries in the FUSE filesystem (1 hour).
const TTL_BACKUPS: Duration = Duration::from_secs(3_600);
/// Time-to-live (TTL) for other cache entries in the FUSE filesystem (over 11 days).
const TTL_REST: Duration = Duration::from_secs(1_000_000);

/// The default cache size for the FUSE filesystem layer (number of elements).
const CACHE_SIZE: usize = 2048;
/// The default creation time for filesystem elements (`UNIX_EPOCH`).
const CREATE_TIME: SystemTime = UNIX_EPOCH;

#[derive(PartialEq, Default, Debug, Clone)]
/// Represents a cached element in the FUSE filesystem.
struct CacheElement {
    /// The path of the cached element.
    pub path: PathBuf,
    /// The inode number of the parent directory.
    pub parent_ino: u64,
}

/// The root element of the FUSE filesystem cache.
static ROOT_ELEMENT: LazyLock<CacheElement> = LazyLock::new(|| CacheElement {
    path: PathBuf::from("/"),
    parent_ino: 0,
});

#[derive(Clone, Debug)]
/// File attributes for a `BackupPC` file in the FUSE filesystem.
pub struct BackupPCFileAttribute {
    /// The name of the file (as bytes).
    pub name: Vec<u8>,
    /// The file attributes (permissions, size, etc.).
    pub attr: FileAttr,
}

impl BackupPCFileAttribute {
    /// Creates a new `BackupPCFileAttribute` from a file manifest and child inode number.
    ///
    /// # Arguments
    ///
    /// * `file` - The file manifest to extract attributes from.
    /// * `child_ino` - The inode number for the child file.
    ///
    /// # Returns
    ///
    /// A new `BackupPCFileAttribute` instance with the extracted attributes.
    pub fn from_file_attribute(file: FileManifest, child_ino: u64) -> Self {
        let mtime = u64::try_from(file.last_modified()).unwrap_or_default();
        let ctime = file
            .stats
            .as_ref()
            .map(|stats| stats.created)
            .and_then(|x| u64::try_from(x).ok())
            .unwrap_or_default();
        let atime = file
            .stats
            .as_ref()
            .map(|stats| stats.last_read)
            .and_then(|x| u64::try_from(x).ok())
            .unwrap_or_default();
        let nlink = file
            .stats
            .as_ref()
            .map(|stats| stats.nlink)
            .and_then(|x| u32::try_from(x).ok())
            .unwrap_or_default();
        let uid = file
            .stats
            .as_ref()
            .map(|stats| stats.owner_id)
            .unwrap_or_default();
        let gid = file
            .stats
            .as_ref()
            .map(|stats| stats.group_id)
            .unwrap_or_default();
        let size = file.size();
        let file_mode = file.file_mode();
        let mode = file.mode();

        BackupPCFileAttribute {
            name: file.path,
            attr: FileAttr {
                ino: child_ino,
                size,
                blocks: size / 512,
                blksize: 512,
                atime: UNIX_EPOCH + Duration::from_secs(atime),
                mtime: UNIX_EPOCH + Duration::from_secs(mtime),
                ctime: UNIX_EPOCH + Duration::from_secs(ctime),
                crtime: UNIX_EPOCH + Duration::from_secs(ctime),
                kind: match file_mode {
                    FileManifestType::Symlink => FileType::Symlink,
                    FileManifestType::CharacterDevice => FileType::CharDevice,
                    FileManifestType::BlockDevice => FileType::BlockDevice,
                    FileManifestType::Directory => FileType::Directory,
                    FileManifestType::Fifo => FileType::NamedPipe,
                    FileManifestType::Socket => FileType::Socket,
                    _ => FileType::RegularFile,
                },
                perm: (mode & 0o777) as u16,
                nlink,
                uid,
                gid,
                rdev: 0,
                flags: 0,
            },
        }
    }
}

/// File attributes for the root element in the FUSE filesystem.
const ROOT_ELEMENT_ATTR: FileAttr = FileAttr {
    ino: 1,
    size: 0,
    blocks: 0,
    blksize: 0,
    atime: CREATE_TIME,
    mtime: CREATE_TIME,
    ctime: CREATE_TIME,
    crtime: CREATE_TIME,
    kind: FileType::Directory,
    perm: 0o755,
    nlink: 1,
    uid: 1,
    gid: 1,
    rdev: 0,
    flags: 0,
};

/// Represents an opened file in the FUSE filesystem.
pub struct OpenedFile {
    /// The current offset within the file.
    pub offset: i64,
    /// The async reader for the file content.
    pub reader: Pin<Box<dyn AsyncBufRead + Send + Sync>>,
}

/// Internal state for the Woodstock FUSE filesystem implementation.
struct WoodstockFileSystemInner {
    /// The Woodstock view for accessing backup data.
    view: WoodstockView,
    /// Mapping from inode numbers to cached elements.
    inodes: HashMap<u64, CacheElement>,
    /// LRU cache for file attributes.
    cache: LruCache<u64, Vec<BackupPCFileAttribute>>,
    /// Mapping from inode numbers to opened files.
    opened: HashMap<u64, OpenedFile>,
    /// The prefix path for the filesystem root.
    prefix_path: PathBuf,
}

impl WoodstockFileSystemInner {
    /// Creates a new `WoodstockFileSystemInner` with the given configuration and prefix path.
    ///
    /// # Arguments
    ///
    /// * `config` - The Woodstock configuration for the filesystem.
    /// * `prefix_path` - The root path for the filesystem.
    ///
    /// # Returns
    ///
    /// A new `WoodstockFileSystemInner` instance.
    pub fn new(config: &Configuration, prefix_path: &Path) -> Self {
        WoodstockFileSystemInner {
            inodes: HashMap::new(),
            view: WoodstockView::new(config),
            cache: LruCache::new(NonZeroUsize::new(CACHE_SIZE).unwrap()),
            opened: HashMap::new(),
            prefix_path: prefix_path.to_path_buf(),
        }
    }

    /// Joins the given path with the filesystem's prefix path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to join with the prefix.
    ///
    /// # Returns
    ///
    /// The joined path as a `PathBuf`.
    fn join_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            let relative_path = path.strip_prefix("/").unwrap_or(path);
            self.prefix_path.join(relative_path)
        } else {
            self.prefix_path.join(path)
        }
    }

    /// Generates a new inode number for a given cache element using a hash of its path.
    ///
    /// # Arguments
    ///
    /// * `elt` - The cache element for which to generate an inode number.
    ///
    /// # Returns
    ///
    /// The generated inode number as a `u64`.
    fn generate_new_ino(&self, elt: &CacheElement) -> u64 {
        let mut hasher = XxHash64::with_seed(0);
        let key = path_to_vec(&elt.path);
        hasher.write(&key);
        let mut hash = hasher.finish();

        // Vérifiez si l'ino est déjà utilisé, si oui, utilisez le sondage quadratique pour trouver un ino libre
        let mut probe = 1;
        while self.inodes.contains_key(&hash) {
            if self.inodes.get(&hash).unwrap_or(&CacheElement::default()) == elt {
                return hash;
            }

            hash += probe * probe;
            probe += 1;
        }

        hash
    }

    /// Lists all files for a given inode and path, returning their attributes.
    ///
    /// # Arguments
    ///
    /// * `ino` - The inode number to list files for.
    /// * `path` - The path to list files in.
    ///
    /// # Returns
    ///
    /// A vector of `BackupPCFileAttribute` for each file found.
    async fn list_files(&mut self, ino: u64, path: &Path) -> Result<Vec<BackupPCFileAttribute>> {
        // Concatenate the path with the prefix path
        let absolute_path = self.join_path(path);
        let files = self.view.list(&absolute_path).await?;

        let result = files
            .into_iter()
            .map(move |file| {
                let path = path.join(file.path());

                let key = CacheElement {
                    path,
                    parent_ino: ino,
                };
                let child_ino = self.generate_new_ino(&key);

                self.inodes.insert(child_ino, key);

                BackupPCFileAttribute::from_file_attribute(file, child_ino)
            })
            .collect();

        Ok(result)
    }

    /// Lists all attributes for a given inode.
    ///
    /// # Arguments
    ///
    /// * `ino` - The inode number to list attributes for.
    ///
    /// # Returns
    ///
    /// A vector of `BackupPCFileAttribute` for the inode.
    async fn list_attributes(&mut self, ino: u64) -> Result<Vec<BackupPCFileAttribute>> {
        let binding = (*ROOT_ELEMENT).clone();
        let cache_element = match ino {
            0 => {
                return Ok(vec![BackupPCFileAttribute {
                    name: b"..".to_vec(),
                    attr: ROOT_ELEMENT_ATTR,
                }])
            }
            1 => Some(&binding),
            _ => self.inodes.get(&ino),
        }
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "No value"))?;

        let path = cache_element.path.clone();

        match self.list_files(ino, &path).await {
            Ok(files) => Ok(files),
            Err(err) => {
                eprintln!("Error listing files of {path:?}: {err}");
                Err(err)
            }
        }
    }

    /// Lists all attributes for a given inode, using the cache if available.
    ///
    /// # Arguments
    ///
    /// * `ino` - The inode number to list attributes for.
    ///
    /// # Returns
    ///
    /// A vector of `BackupPCFileAttribute` for the inode.
    async fn list_attributes_with_cache(&mut self, ino: u64) -> Result<Vec<BackupPCFileAttribute>> {
        if let Some(cached_result) = self.cache.get(&ino) {
            return Ok(cached_result.clone());
        }

        let mut result = self.list_attributes(ino).await?;
        result.sort_by(|a, b| a.name.cmp(&b.name));
        self.cache.put(ino, result.clone());

        Ok(result)
    }

    /// Gets the file attribute and its duration for a given inode and file name.
    ///
    /// # Arguments
    ///
    /// * `ino` - The inode number.
    /// * `name` - The file name as an `OsStr`.
    ///
    /// # Returns
    ///
    /// An option containing the duration and file attribute if found.
    async fn get_file_attr(&mut self, ino: u64, name: &OsStr) -> Option<(Duration, FileAttr)> {
        let binding = (*ROOT_ELEMENT).clone();
        let cache_element = match ino {
            1 => Some(&binding),
            _ => self.inodes.get(&ino),
        }?;

        let duration = match cache_element.path.components().count() {
            0 => TTL_HOST,
            1 => TTL_BACKUPS,
            _ => TTL_REST,
        };

        let name = osstr_to_vec(name);
        let attributes = self.list_attributes_with_cache(ino).await;
        let attribute = match attributes {
            Ok(attrs) => attrs.into_iter().find(|attr| attr.name == name),
            Err(_) => None,
        };

        attribute.map(|attr| (duration, attr.attr))
    }

    /// Fills a reply with directory entries from a list of files for a given inode.
    ///
    /// # Arguments
    ///
    /// * `reply` - The reply object to fill.
    /// * `ino` - The inode number.
    /// * `offset` - The offset to start reading from.
    ///
    /// # Returns
    ///
    /// An empty result on success.
    async fn fill_reply_from_files(
        &mut self,
        reply: &mut ReplyDirectory,
        ino: u64,
        offset: i64,
    ) -> Result<()> {
        let elements = self.list_attributes_with_cache(ino).await?;

        // Add the "." and ".." entries
        if ino != 1 && offset == 0 {
            let element = self.inodes.get(&ino);
            if let Some(parent) = element {
                let _ = reply.add(ino, 1, FileType::Directory, ".");
                let _ = reply.add(parent.parent_ino, 2, FileType::Directory, "..");
            }
        }

        let offset = usize::try_from(offset)?;
        for (current_offset, cache_element) in elements.iter().enumerate() {
            let current_offset = current_offset + 2;
            if current_offset <= offset {
                continue;
            }

            let name = vec_to_osstr(&cache_element.name);

            debug!(
                "Adding entry {} to ino {}, offset: {}, kind: {:?}",
                name.to_string_lossy(),
                cache_element.attr.ino,
                current_offset,
                cache_element.attr.kind,
            );
            let result = reply.add(
                cache_element.attr.ino,
                current_offset as i64,
                cache_element.attr.kind,
                name,
            );
            if result {
                break;
            }
        }
        Ok(())
    }

    /// Gets the file attribute and its duration for a given inode.
    ///
    /// # Arguments
    ///
    /// * `ino` - The inode number.
    ///
    /// # Returns
    ///
    /// An option containing the duration and file attribute if found.
    async fn get_attr(&mut self, ino: u64) -> Option<(Duration, FileAttr)> {
        let binding = (*ROOT_ELEMENT).clone();
        let cache_element = match ino {
            1 => Some(&binding),
            _ => self.inodes.get(&ino),
        }?;

        let duration = match cache_element.path.components().count() {
            0 => TTL_HOST,
            1 => TTL_BACKUPS,
            _ => TTL_REST,
        };

        let parent_ino = cache_element.parent_ino;

        let attributes = self.list_attributes_with_cache(parent_ino).await;
        let attribute = match attributes {
            Ok(attrs) => attrs.into_iter().find(|attr| attr.attr.ino == ino),
            Err(_) => None,
        };

        attribute.map(|attr| (duration, attr.attr))
    }

    /// Generates a random file handle for opened files.
    ///
    /// # Returns
    ///
    /// A random `u64` value not currently used as a file handle.
    fn generate_file_handle(&self) -> u64 {
        // Random file handle not used in opened files
        loop {
            let handle = rand::random::<u64>();
            if !self.opened.contains_key(&handle) {
                return handle;
            }
        }
    }

    /// Creates an async reader for the file associated with the given inode.
    ///
    /// # Arguments
    ///
    /// * `ino` - The inode number to create a reader for.
    ///
    /// # Returns
    ///
    /// An async buffered reader for the file content.
    async fn create_reader(&mut self, ino: u64) -> Result<impl AsyncBufRead> {
        let binding = (*ROOT_ELEMENT).clone();
        let cache_element = match ino {
            1 => Some(&binding),
            _ => self.inodes.get(&ino),
        }
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to get filename",
        ))?;

        let path = cache_element.path.clone();
        let path = self.join_path(&path);

        match self.view.read_file(&path).await {
            Ok(reader) => Ok(reader),
            Err(err) => {
                eprintln!("Can't open the file {path:?}: {err}");
                Err(err)
            }
        }
    }

    /// Reads the symbolic link target for the given inode.
    ///
    /// # Arguments
    ///
    /// * `ino` - The inode number to read the link for.
    ///
    /// # Returns
    ///
    /// The target of the symbolic link as a byte vector.
    async fn read_link(&mut self, ino: u64) -> Result<Vec<u8>> {
        let binding = (*ROOT_ELEMENT).clone();
        let cache_element = match ino {
            1 => Some(&binding),
            _ => self.inodes.get(&ino),
        }
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to get filename",
        ))?;

        let path = &cache_element.path;
        match self.view.get_attribute(&self.prefix_path.join(path)).await {
            Ok(manifest) => Ok(manifest.symlink.clone()),
            Err(err) => {
                eprintln!("Can't read the link {path:?}: {err}");
                Err(err)
            }
        }
    }

    /// Opens a file for the given inode and returns a file handle.
    ///
    /// # Arguments
    ///
    /// * `ino` - The inode number to open.
    ///
    /// # Returns
    ///
    /// The file handle as a `u64`.
    async fn open(&mut self, ino: u64) -> Result<u64> {
        let reader = self.create_reader(ino).await?;
        let fh = self.generate_file_handle();
        self.opened.insert(
            fh,
            OpenedFile {
                offset: 0,
                reader: Box::pin(reader),
            },
        );

        Ok(fh)
    }

    /// Releases the file handle, closing the associated file.
    ///
    /// # Arguments
    ///
    /// * `fh` - The file handle to release.
    fn release(&mut self, fh: u64) {
        self.opened.remove(&fh);
    }

    /// Reads data from a file given its inode and file handle.
    ///
    /// # Arguments
    ///
    /// * `ino` - The inode number.
    /// * `fh` - The file handle.
    /// * `offset` - The offset to start reading from.
    /// * `size` - The number of bytes to read.
    ///
    /// # Returns
    ///
    /// The read data as a byte vector.
    async fn read_ino(&mut self, ino: u64, fh: u64, offset: i64, size: u32) -> Result<Vec<u8>> {
        let opened_file = self
            .opened
            .get(&fh)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "File not opened"))?;

        // If the offset is lesser than the current offset, we need to reset the reader
        if offset < opened_file.offset {
            let reader = self.create_reader(ino).await?;

            let opened_file = self.opened.get_mut(&fh).unwrap();
            opened_file.reader = Box::pin(reader);
            opened_file.offset = 0;
        }

        let opened_file = self.opened.get_mut(&fh).unwrap();

        // If the offset is greater that the current offset, we need to fast forward (by reading data by 32k chunk )
        if offset > opened_file.offset {
            let mut buffer = vec![0; 32 * 1024];
            let mut remaining = offset - opened_file.offset;

            while remaining > 0 {
                let to_read = std::cmp::min(remaining, buffer.len() as i64);
                let to_read = usize::try_from(to_read)?;
                let size: usize = opened_file.reader.read(&mut buffer[..to_read]).await?;
                remaining -= size as i64;
                if size == 0 {
                    info!("End of file reached");
                    break;
                }
            }
            opened_file.offset = offset;
        }

        // Read the data
        let mut reader = opened_file.reader.as_mut();
        let mut buffer = vec![0; size as usize];

        let size = reader.read(&mut buffer).await?;
        opened_file.offset += size as i64;

        // Reduce the size of the buffer to the actual size read
        buffer.truncate(size);

        Ok(buffer)
    }
}

/// The main FUSE filesystem struct for Woodstock backups.
pub struct WoodstockFileSystem {
    /// The inner state of the filesystem, protected by a mutex for thread safety.
    inner: Arc<Mutex<WoodstockFileSystemInner>>,
}

impl WoodstockFileSystem {
    /// Creates a new `WoodstockFileSystem` instance with the given configuration and prefix path.
    ///
    /// # Arguments
    ///
    /// * `config` - The Woodstock configuration for the filesystem.
    /// * `prefix_path` - The root path for the filesystem.
    ///
    /// # Returns
    ///
    /// A new `WoodstockFileSystem` instance.
    pub fn new(config: &Configuration, prefix_path: &Path) -> Self {
        WoodstockFileSystem {
            inner: Arc::new(Mutex::new(WoodstockFileSystemInner::new(
                config,
                prefix_path,
            ))),
        }
    }
}

impl Filesystem for WoodstockFileSystem {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let inner = Arc::clone(&self.inner);
        let name = name.to_os_string();

        tokio::spawn(async move {
            let mut inner = inner.lock().await;
            let attr = inner.get_file_attr(parent, &name).await;
            debug!("Lookup parent: {parent}, name: {name:?}, attr: {attr:?}");

            match attr {
                Some((ttl, attr)) => reply.entry(&ttl, &attr, 0),
                None => reply.error(ENOENT),
            }
        });
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
        let inner = Arc::clone(&self.inner);

        tokio::spawn(async move {
            let mut inner = inner.lock().await;
            let attr = inner.get_attr(ino).await;
            debug!("Getattr ino: {ino}, attr: {attr:?}");

            match attr {
                Some((ttl, attr)) => reply.attr(&ttl, &attr),
                None => reply.error(ENOENT),
            }
        });
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        debug!("Readdir ino: {ino}, offset: {offset}");

        let inner = Arc::clone(&self.inner);

        tokio::spawn(async move {
            let mut inner = inner.lock().await;
            // List host and add it to the cache
            match inner.fill_reply_from_files(&mut reply, ino, offset).await {
                Ok(()) => {
                    reply.ok();
                }
                Err(e) => {
                    eprintln!("Error reading dir {ino}: {e}");
                    reply.error(ENOENT);
                }
            }
        });
    }

    fn readlink(&mut self, _req: &Request<'_>, ino: u64, reply: ReplyData) {
        let inner = Arc::clone(&self.inner);

        tokio::spawn(async move {
            let mut inner = inner.lock().await;

            let link_to = inner.read_link(ino).await;
            debug!("Readlink ino: {ino}, attr: {link_to:?}");

            match link_to {
                Ok(data) => reply.data(&data),
                Err(err) => {
                    eprintln!("Error reading link ino {ino}: {err}");
                    reply.error(ENOENT);
                }
            }
        });
    }

    fn open(&mut self, _req: &Request<'_>, ino: u64, _flags: i32, reply: ReplyOpen) {
        let inner = Arc::clone(&self.inner);

        tokio::spawn(async move {
            let mut inner = inner.lock().await;

            match inner.open(ino).await {
                Ok(fh) => reply.opened(fh, 0),
                Err(err) => {
                    eprintln!("Error opening ino {ino}: {err}");
                    reply.error(ENOENT);
                }
            }
        });
    }

    fn read(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        let inner = Arc::clone(&self.inner);

        tokio::spawn(async move {
            let mut inner = inner.lock().await;

            match inner.read_ino(ino, fh, offset, size).await {
                Ok(data) => reply.data(&data),
                Err(err) => {
                    eprintln!("Error reading ino {ino}: {err}");
                    reply.error(ENOENT);
                }
            }
        });
    }

    fn release(
        &mut self,
        _req: &Request<'_>,
        _ino: u64,
        fh: u64,
        _flags: i32,
        _lock_owner: Option<u64>,
        _flush: bool,
        reply: ReplyEmpty,
    ) {
        let inner = Arc::clone(&self.inner);

        tokio::spawn(async move {
            let mut inner = inner.lock().await;

            inner.release(fh);
            reply.ok();
        });
    }
}

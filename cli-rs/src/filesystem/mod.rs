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
use woodstock::config::Context;
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

const TTL_HOST: Duration = Duration::from_secs(86_400);
const TTL_BACKUPS: Duration = Duration::from_secs(3_600);
const TTL_REST: Duration = Duration::from_secs(1_000_000);

const CACHE_SIZE: usize = 2048;

const CREATE_TIME: SystemTime = UNIX_EPOCH;

#[derive(PartialEq, Default, Debug, Clone)]
struct CacheElement {
    pub path: PathBuf,
    pub parent_ino: u64,
}

static ROOT_ELEMENT: LazyLock<CacheElement> = LazyLock::new(|| CacheElement {
    path: PathBuf::from("/"),
    parent_ino: 0,
});

#[derive(Clone, Debug)]
pub struct BackupPCFileAttribute {
    pub name: Vec<u8>,
    pub attr: FileAttr,
}

impl BackupPCFileAttribute {
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

pub struct OpenedFile {
    pub offset: i64,
    pub reader: Pin<Box<dyn AsyncBufRead + Send + Sync>>,
}

struct WoodstockFileSystemInner {
    view: WoodstockView,
    inodes: HashMap<u64, CacheElement>,
    cache: LruCache<u64, Vec<BackupPCFileAttribute>>,
    opened: HashMap<u64, OpenedFile>,
    prefix_path: PathBuf,
}

impl WoodstockFileSystemInner {
    pub fn new(ctxt: &Context, prefix_path: &Path) -> Self {
        WoodstockFileSystemInner {
            inodes: HashMap::new(),
            view: WoodstockView::new(ctxt),
            cache: LruCache::new(NonZeroUsize::new(CACHE_SIZE).unwrap()),
            opened: HashMap::new(),
            prefix_path: prefix_path.to_path_buf(),
        }
    }

    fn join_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            let relative_path = path.strip_prefix("/").unwrap_or(path);
            self.prefix_path.join(relative_path)
        } else {
            self.prefix_path.join(path)
        }
    }

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
                eprintln!("Error listing files of {:?}: {}", path, err);
                Err(err)
            }
        }
    }

    async fn list_attributes_with_cache(&mut self, ino: u64) -> Result<Vec<BackupPCFileAttribute>> {
        if let Some(cached_result) = self.cache.get(&ino) {
            return Ok(cached_result.clone());
        }

        let mut result = self.list_attributes(ino).await?;
        result.sort_by(|a, b| a.name.cmp(&b.name));
        self.cache.put(ino, result.clone());

        Ok(result)
    }

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

    fn generate_file_handle(&self) -> u64 {
        // Random file handle not used in opened files
        loop {
            let handle = rand::random::<u64>();
            if !self.opened.contains_key(&handle) {
                return handle;
            }
        }
    }

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
                eprintln!("Can't open the file {:?}: {}", path, err);
                Err(err)
            }
        }
    }

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
                eprintln!("Can't read the link {:?}: {}", path, err);
                Err(err)
            }
        }
    }

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

    fn release(&mut self, fh: u64) {
        self.opened.remove(&fh);
    }

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

pub struct WoodstockFileSystem {
    inner: Arc<Mutex<WoodstockFileSystemInner>>,
}

impl WoodstockFileSystem {
    pub fn new(ctxt: &Context, prefix_path: &Path) -> Self {
        WoodstockFileSystem {
            inner: Arc::new(Mutex::new(WoodstockFileSystemInner::new(ctxt, prefix_path))),
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

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
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

use std::collections::HashMap;

use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use woodstock::{
    FileManifest, FileManifestAcl, FileManifestAclQualifier, FileManifestStat, FileManifestType,
    FileManifestXAttr,
};

use crate::graphql::scalars::{BigIntScalar, BufferScalar};

#[derive(
    async_graphql::Enum, Copy, Eq, PartialEq, Debug, Clone, Serialize, Deserialize, ToSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum FileManifestTypeDto {
    RegularFile = 0,
    Symlink = 1,
    Directory = 2,
    BlockDevice = 3,
    CharacterDevice = 4,
    Fifo = 5,
    Socket = 6,
    Unknown = 99,
}

impl From<FileManifestType> for FileManifestTypeDto {
    fn from(status: FileManifestType) -> Self {
        match status {
            FileManifestType::RegularFile => FileManifestTypeDto::RegularFile,
            FileManifestType::Symlink => FileManifestTypeDto::Symlink,
            FileManifestType::Directory => FileManifestTypeDto::Directory,
            FileManifestType::BlockDevice => FileManifestTypeDto::BlockDevice,
            FileManifestType::CharacterDevice => FileManifestTypeDto::CharacterDevice,
            FileManifestType::Fifo => FileManifestTypeDto::Fifo,
            FileManifestType::Socket => FileManifestTypeDto::Socket,
            FileManifestType::Unknown => FileManifestTypeDto::Unknown,
        }
    }
}

#[derive(
    async_graphql::Enum, Copy, Eq, PartialEq, Debug, Clone, Serialize, Deserialize, ToSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum FileManifestAclQualifierDto {
    Undefined = 0,
    UserObj = 1,
    GroupObj = 2,
    Other = 3,
    UserId = 4,
    GroupId = 5,
    Mask = 6,
}

impl From<FileManifestAclQualifier> for FileManifestAclQualifierDto {
    fn from(qualifier: FileManifestAclQualifier) -> Self {
        match qualifier {
            FileManifestAclQualifier::Undefined => FileManifestAclQualifierDto::Undefined,
            FileManifestAclQualifier::UserObj => FileManifestAclQualifierDto::UserObj,
            FileManifestAclQualifier::GroupObj => FileManifestAclQualifierDto::GroupObj,
            FileManifestAclQualifier::Other => FileManifestAclQualifierDto::Other,
            FileManifestAclQualifier::UserId => FileManifestAclQualifierDto::UserId,
            FileManifestAclQualifier::GroupId => FileManifestAclQualifierDto::GroupId,
            FileManifestAclQualifier::Mask => FileManifestAclQualifierDto::Mask,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct FileStat {
    pub owner_id: u32,
    pub group_id: u32,
    pub size: BigIntScalar,
    pub compressed_size: BigIntScalar,
    pub last_read: i64,
    pub last_modified: i64,
    pub created: i64,
    pub mode: u32,
    pub r#type: FileManifestTypeDto,
    pub dev: BigIntScalar,
    pub rdev: BigIntScalar,
    pub ino: BigIntScalar,
    pub nlink: BigIntScalar,
}

impl From<&FileManifestStat> for FileStat {
    fn from(stat: &FileManifestStat) -> Self {
        Self {
            owner_id: stat.owner_id,
            group_id: stat.group_id,
            size: BigIntScalar(stat.size),
            compressed_size: BigIntScalar(stat.compressed_size),
            last_read: stat.last_read,
            last_modified: stat.last_modified,
            created: stat.created,
            mode: stat.mode,
            r#type: stat.file_type().into(),
            dev: BigIntScalar(stat.dev),
            rdev: BigIntScalar(stat.rdev),
            ino: BigIntScalar(stat.ino),
            nlink: BigIntScalar(stat.nlink),
        }
    }
}

impl From<FileManifestStat> for FileStat {
    fn from(stat: FileManifestStat) -> Self {
        Self::from(&stat)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct FileAcl {
    pub qualifier: FileManifestAclQualifierDto,
    pub id: u32,
    pub perm: u32,
}

impl From<&FileManifestAcl> for FileAcl {
    fn from(acl: &FileManifestAcl) -> Self {
        Self {
            qualifier: acl.qualifier().into(),
            id: acl.id,
            perm: acl.perm,
        }
    }
}

impl From<FileManifestAcl> for FileAcl {
    fn from(acl: FileManifestAcl) -> Self {
        Self::from(&acl)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct FileXAttr {
    pub key: BufferScalar,
    pub value: BufferScalar,
}

impl From<&FileManifestXAttr> for FileXAttr {
    fn from(dto: &FileManifestXAttr) -> Self {
        Self {
            key: BufferScalar(dto.key.clone()),
            value: BufferScalar(dto.value.clone()),
        }
    }
}
impl From<FileManifestXAttr> for FileXAttr {
    fn from(dto: FileManifestXAttr) -> Self {
        // Consommer directement au lieu de cloner via référence
        Self {
            key: BufferScalar(dto.key),
            value: BufferScalar(dto.value),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, SimpleObject)]
#[graphql(complex)]
#[serde(rename_all = "camelCase")]
pub struct FileDescription {
    pub path: BufferScalar,
    pub stats: Option<FileStat>,
    pub symlink: BufferScalar,
    pub xattr: Vec<FileXAttr>,
    pub acl: Vec<FileAcl>,
    pub chunks: Vec<BufferScalar>,
    pub hash: BufferScalar,
    pub metadata: HashMap<String, BufferScalar>,
}

impl From<&FileManifest> for FileDescription {
    fn from(manifest: &FileManifest) -> Self {
        Self {
            path: BufferScalar(manifest.path.clone()),
            stats: manifest.stats.map(|s| s.into()),
            symlink: BufferScalar(manifest.symlink.clone()),
            xattr: manifest.xattr.iter().map(|x| x.into()).collect(),
            acl: manifest.acl.iter().map(|a| a.into()).collect(),
            chunks: manifest.chunks.iter().cloned().map(BufferScalar).collect(),
            hash: BufferScalar(manifest.hash.clone()),
            metadata: manifest
                .metadata
                .iter()
                .map(|(k, v)| (k.clone(), BufferScalar(v.clone())))
                .collect(),
        }
    }
}
impl From<FileManifest> for FileDescription {
    fn from(manifest: FileManifest) -> Self {
        Self {
            path: BufferScalar(manifest.path),
            stats: manifest.stats.map(|s| s.into()),
            symlink: BufferScalar(manifest.symlink),
            xattr: manifest.xattr.into_iter().map(|x| x.into()).collect(),
            acl: manifest.acl.into_iter().map(|a| a.into()).collect(),
            chunks: manifest.chunks.into_iter().map(BufferScalar).collect(),
            hash: BufferScalar(manifest.hash),
            metadata: manifest
                .metadata
                .into_iter()
                .map(|(k, v)| (k, BufferScalar(v)))
                .collect(),
        }
    }
}

impl From<String> for FileDescription {
    fn from(s: String) -> Self {
        use super::FileManifestTypeDto;
        Self {
            path: BufferScalar(s.into()),
            stats: Some(FileStat {
                owner_id: 0,
                group_id: 0,
                size: BigIntScalar(0),
                compressed_size: BigIntScalar(0),
                last_read: 0,
                last_modified: 0,
                created: 0,
                mode: 0x755,
                r#type: FileManifestTypeDto::Directory,
                dev: BigIntScalar(0),
                rdev: BigIntScalar(0),
                ino: BigIntScalar(0),
                nlink: BigIntScalar(0),
            }),
            symlink: BufferScalar(Vec::new()),
            xattr: Vec::new(),
            acl: Vec::new(),
            chunks: Vec::new(),
            hash: BufferScalar(Vec::new()),
            metadata: HashMap::new(),
        }
    }
}

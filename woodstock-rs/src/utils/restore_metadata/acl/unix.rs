//! Unix implementation for ACL operations.
//!
//! Uses the `posix_acl` crate to interface with the underlying POSIX Access
//! Control List system.

use std::path::Path;

use eyre::Result;
use posix_acl::{PosixACL, Qualifier};

use crate::{FileManifestAcl, FileManifestAclQualifier};

/// Reads the Access Control Lists for a file on Unix systems.
///
/// # Errors
/// Returns an error if the file does not exist, the process lacks
/// permission to read its ACLs, or the underlying POSIX ACL calls fail.
pub fn read_acl(file: &Path) -> Result<Vec<FileManifestAcl>> {
    let acls: PosixACL = PosixACL::read_acl(file)?;
    let acls = acls.entries();

    let acl = acls
        .iter()
        .map(|entry| {
            let mut id = 0;
            let qualifier = match entry.qual {
                Qualifier::Undefined => FileManifestAclQualifier::Undefined,
                Qualifier::UserObj => FileManifestAclQualifier::UserObj,
                Qualifier::User(user) => {
                    id = user;
                    FileManifestAclQualifier::UserId
                }
                Qualifier::GroupObj => FileManifestAclQualifier::GroupObj,
                Qualifier::Group(group) => {
                    id = group;
                    FileManifestAclQualifier::GroupId
                }
                Qualifier::Mask => FileManifestAclQualifier::Mask,
                Qualifier::Other => FileManifestAclQualifier::Other,
            };

            FileManifestAcl {
                qualifier: qualifier as i32,
                id,
                perm: entry.perm,
            }
        })
        .collect();

    Ok(acl)
}

/// Restores Access Control Lists to a file on Unix systems.
///
/// First reads the current ACLs of the file and then modifies them, rather
/// than creating an entirely new ACL list — this preserves any
/// system-specific ACL entries that might not be part of the saved ACLs.
///
/// # Errors
/// Returns an error if the file does not exist, the process lacks
/// permission to modify its ACLs, or the underlying POSIX ACL calls fail.
pub fn restore_acl(file: &Path, acls: &[FileManifestAcl]) -> Result<()> {
    let mut acls_writer: PosixACL = PosixACL::read_acl(file)?;

    for acl in acls {
        let qualifier = match acl.qualifier() {
            FileManifestAclQualifier::Undefined => Qualifier::Undefined,
            FileManifestAclQualifier::UserObj => Qualifier::UserObj,
            FileManifestAclQualifier::UserId => {
                let user = acl.id;
                Qualifier::User(user)
            }
            FileManifestAclQualifier::GroupObj => Qualifier::GroupObj,
            FileManifestAclQualifier::GroupId => {
                let group = acl.id;
                Qualifier::Group(group)
            }
            FileManifestAclQualifier::Mask => Qualifier::Mask,
            FileManifestAclQualifier::Other => Qualifier::Other,
        };

        acls_writer.set(qualifier, acl.perm);
    }

    acls_writer.write_acl(file)?;
    Ok(())
}

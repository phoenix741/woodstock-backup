//! Unix implementation for ACL operations.
//!
//! Uses the `posix_acl` crate to interface with the underlying POSIX Access
//! Control List system.

use std::collections::HashSet;
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
/// First reads the current ACLs of the file and then modifies them, rather than creating an
/// entirely new ACL list — this preserves the base `UserObj`/`GroupObj`/`Other`/`Mask`
/// entries (always merge-set, never removed: every valid ACL requires them, and they aren't
/// individually "revocable" the way a named grant is) plus any other system-specific entry
/// not covered by named users/groups. Named `User`/`Group` entries are restored
/// declaratively, though: one present on the destination but absent from `acls` is a grant
/// that was revoked since the backup was taken, and is removed rather than left in place —
/// otherwise a "restore" could end up more permissive than the snapshot it's restoring.
///
/// # Errors
/// Returns an error if the file does not exist, the process lacks
/// permission to modify its ACLs, or the underlying POSIX ACL calls fail.
pub fn restore_acl(file: &Path, acls: &[FileManifestAcl]) -> Result<()> {
    let mut acls_writer: PosixACL = PosixACL::read_acl(file)?;

    let mut desired_users = HashSet::new();
    let mut desired_groups = HashSet::new();
    for acl in acls {
        match acl.qualifier() {
            FileManifestAclQualifier::UserId => {
                desired_users.insert(acl.id);
            }
            FileManifestAclQualifier::GroupId => {
                desired_groups.insert(acl.id);
            }
            _ => {}
        }
    }

    for entry in acls_writer.entries() {
        match entry.qual {
            Qualifier::User(id) if !desired_users.contains(&id) => {
                acls_writer.remove(Qualifier::User(id));
            }
            Qualifier::Group(id) if !desired_groups.contains(&id) => {
                acls_writer.remove(Qualifier::Group(id));
            }
            _ => {}
        }
    }

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

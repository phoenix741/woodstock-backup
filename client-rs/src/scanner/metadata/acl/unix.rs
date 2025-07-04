/// Unix implementation for ACL operations.
///
/// This module provides concrete implementations for reading and restoring
/// POSIX Access Control Lists (ACLs) on Unix systems when the 'acl' feature is enabled.
/// It uses the `posix_acl` crate to interface with the underlying ACL system.
use std::path::Path;

use eyre::Result;
use posix_acl::{PosixACL, Qualifier};
use woodstock::{FileManifestAcl, FileManifestAclQualifier};

/// Reads the Access Control Lists for a file on Unix systems.
///
/// This function reads the POSIX ACLs from a file and converts them to the
/// application's internal `FileManifestAcl` format for storage.
///
/// # Arguments
/// * `file` - Path to the file to read ACLs from.
///
/// # Returns
/// * `Result<Vec<FileManifestAcl>>` - A vector of ACL entries if successful, or an error.
///
/// # Errors
/// Returns an error if:
/// * The file does not exist
/// * The process lacks permissions to read the file's ACLs
/// * The underlying POSIX ACL functions fail
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

            // Create a FileManifestAcl object representing this ACL entry
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
/// This function takes a list of `FileManifestAcl` entries and applies them
/// to the specified file, restoring its access permissions to match the saved state.
///
/// # Arguments
/// * `file` - Path to the file to restore ACLs to.
/// * `acls` - Vector of ACL entries to restore to the file.
///
/// # Returns
/// * `Result<()>` - Success or error result.
///
/// # Errors
/// Returns an error if:
/// * The file does not exist
/// * The process lacks permissions to modify the file's ACLs
/// * The underlying POSIX ACL functions fail
///
/// # Note
/// This function first reads the current ACLs of the file and then modifies them,
/// rather than creating an entirely new ACL list. This helps preserve any system-specific
/// ACL entries that might not be part of the saved ACLs.
pub fn restore_acl(file: &Path, acls: &[FileManifestAcl]) -> Result<()> {
    use posix_acl::{PosixACL, Qualifier};
    use woodstock::FileManifestAclQualifier;

    let mut acls_writer: PosixACL = PosixACL::read_acl(file)?;

    for acl in acls {
        // Convert from FileManifestAcl qualifier to POSIX Qualifier
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

        // Set the ACL entry with the specified qualifier and permissions
        acls_writer.set(qualifier, acl.perm);
    }

    // Write the modified ACLs back to the file
    acls_writer.write_acl(file)?;
    Ok(())
}

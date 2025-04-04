use std::path::Path;

use crate::{FileManifestAcl, FileManifestAclQualifier};
use eyre::Result;
use posix_acl::{PosixACL, Qualifier};

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

pub fn restore_acl(file: &Path, acls: &[FileManifestAcl]) -> Result<()> {
    use crate::woodstock::FileManifestAclQualifier;
    use posix_acl::{PosixACL, Qualifier};

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

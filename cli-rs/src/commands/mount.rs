use std::path::PathBuf;

use eyre::Result;
use woodstock::config::Context;

use crate::filesystem::WoodstockFileSystem;

pub struct MountOption {
    pub hostname: Option<String>,
    pub backup_number: Option<usize>,
    pub path: Option<String>,
    pub mount_point: String,
}

pub async fn mount(ctxt: &Context, options: &MountOption) -> Result<()> {
    let path = vec![
        options.hostname.clone(),
        options.backup_number.map(|x| x.to_string()),
        options.path.clone(),
    ];
    let path = path
        .into_iter()
        .flatten()
        .collect::<Vec<String>>()
        .join("/");
    let path = format!("/{}", path);

    // Mount path
    println!("Mounting path: {}", options.mount_point);
    if !path.is_empty() {
        println!("Prefix Path: {}", path);
    }

    let path = PathBuf::from(&path);

    let fs = WoodstockFileSystem::new(ctxt, &path);
    let mount_options = [];
    fuser::mount2(fs, &options.mount_point, &mount_options).unwrap();
    Ok(())
}

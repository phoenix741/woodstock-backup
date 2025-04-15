use std::path::Path;

use eyre::Result;
use reflink_copy::reflink;

pub async fn copy_files<T: AsRef<Path>, U: AsRef<Path>>(
    source: T,
    destination: U,
    files: &[&str],
) -> Result<()> {
    for file in files {
        let source_path = source.as_ref().join(file);
        let dest_path = destination.as_ref().join(file);
        if source_path.exists() {
            let reflink_result = reflink(&source_path, &dest_path);
            if reflink_result.is_err() {
                tokio::fs::copy(&source_path, &dest_path).await?;
            }
        }
    }
    Ok(())
}

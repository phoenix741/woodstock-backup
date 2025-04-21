use std::path::Path;

use eyre::Result;
use reflink_copy::reflink;
use uuid::Uuid;

/// Copy a file from the source directory to the destination directory.
/// The file is copied atomically using reflink if supported, otherwise it falls back to a standard copy.
///
/// # Arguments
/// * `source` - The source file path.
/// * `destination` - The destination file path.
///
/// # Returns
/// * `Ok(())` if the file is successfully copied.
/// * `Err(eyre::Report)` if an error occurs during the copy process.
///
/// # Errors
/// Returns an error if:
/// * The source file does not exist.
/// * The destination file cannot be created.
pub async fn copy_file<T: AsRef<Path>, U: AsRef<Path>>(source: T, destination: U) -> Result<()> {
    let source_path = source.as_ref();
    let dest_path = destination.as_ref();

    if !source_path.exists() {
        return Err(eyre::eyre!(
            "Source file does not exist: {}",
            source_path.display()
        ));
    }

    // Create a temporary file path with a unique name
    let temp_filename = format!(
        ".{}.{}",
        dest_path.file_name().unwrap_or_default().to_string_lossy(),
        Uuid::new_v4()
    );
    let temp_path = dest_path.with_file_name(temp_filename);

    // Ensure parent directory exists
    if let Some(parent) = dest_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Copy to the temporary file first
    let reflink_result = reflink(source_path, &temp_path);
    if reflink_result.is_err() {
        tokio::fs::copy(source_path, &temp_path).await?;
    }

    // Atomically rename the temporary file to the destination
    tokio::fs::rename(&temp_path, dest_path).await?;

    Ok(())
}

/// Copies a list of files from the source directory to the destination directory.
/// Files are copied atomically.
///
/// # Arguments
/// * `source` - The source directory.
/// * `destination` - The destination directory.
/// * `files` - A list of file names to copy.
///
/// # Returns
/// * `Ok(())` if all files are successfully copied.
/// * `Err(eyre::Report)` if an error occurs during the copy process.
///
/// # Errors
/// Returns an error if:
/// * The source file does not exist.
/// * The destination file cannot be created.
/// * The file cannot be copied using either reflink or standard copy.
pub async fn copy_files<T: AsRef<Path>, U: AsRef<Path>>(
    source: T,
    destination: U,
    files: &[&str],
) -> Result<()> {
    for file in files {
        let source_path = source.as_ref().join(file);
        let dest_path = destination.as_ref().join(file);
        copy_file(&source_path, &dest_path).await?;
    }
    Ok(())
}

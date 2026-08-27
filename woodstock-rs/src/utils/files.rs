use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

use eyre::Result;
use reflink_copy::reflink;
use uuid::Uuid;

/// Creates/truncates the file at `path` for a full-content rewrite,
/// self-healing a destination stuck at a restrictive mode instead of
/// failing forever. Shared between `archiving::fs_materialize` (wrapped in
/// `spawn_blocking`, since that caller is async) and `client-rs`'s
/// synchronous real-restore path, which calls this directly.
///
/// A pre-existing destination file can be unwritable for legitimate reasons
/// (e.g. git leaves its loose objects/packs at `0o444`) or because an
/// earlier bug wrote a garbage mode onto it (the historical Windows
/// `FILE_ATTRIBUTE_*`-as-POSIX-mode bug) — either way, `open()` fails
/// `EACCES` before the caller's permission-restore step (called right after
/// this) ever runs. Retrying only on that failure (never chmod'ing up
/// front) keeps the common case — a file that's already writable — at zero
/// extra syscalls.
///
/// This always succeeds when the block is a restrictive mode: `chmod` is
/// gated on uid match (or `CAP_FOWNER`), never on the file's own permission
/// bits, and this process is the same one that created every pre-existing
/// destination file here. It correctly still fails if the destination was
/// `chown`'d to another user by an admin — that's a real permission error,
/// not a stale-mode one.
///
/// # Errors
/// Returns an error if the file cannot be opened even after the retry.
pub fn open_for_write_retrying_on_eacces(path: &Path) -> io::Result<File> {
    let mut opts = OpenOptions::new();
    opts.write(true).create(true).truncate(true);

    match opts.open(path) {
        #[cfg(unix)]
        Err(err) if err.kind() == io::ErrorKind::PermissionDenied && path.exists() => {
            use std::os::unix::fs::PermissionsExt;

            // The 0o600 set here never survives on disk: the caller's
            // permission-restore step re-applies the entry's real final
            // mode (e.g. 0o444 for a git object) right after this returns.
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
            opts.open(path)
        }
        other => other,
    }
}

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

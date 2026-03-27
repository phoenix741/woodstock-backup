//! Linux-specific COW copy using `copy_file_range(2)`.
//!
//! `copy_file_range(2)` performs an in-kernel copy. On COW-capable filesystems
//! (btrfs, XFS) and when source and destination reside on the same filesystem,
//! the kernel can satisfy the copy with a metadata-only block reference, saving
//! both time and disk space. Falls back transparently when the syscall returns
//! ENOSYS (old kernel), EBADF (O_APPEND destination, kernel ≥ 5.19), EXDEV
//! (cross-device), or any other non-retryable error.

use std::fs::File as StdFile;
use std::os::unix::io::AsRawFd;
use std::path::Path;

use eyre::{bail, Result};
use tokio::fs::File;
use tokio::io::AsyncSeekExt;

/// In-kernel copy loop. `src_file` and `dst_file` are kept alive for the
/// entire duration so their fds remain valid inside `spawn_blocking`.
fn try_copy_file_range(
    src_file: &StdFile,
    dst_file: &StdFile,
    len: u64,
    dst_offset: u64,
) -> Result<u64> {
    let src_fd = src_file.as_raw_fd();
    let dst_fd = dst_file.as_raw_fd();
    let mut dst_off = i64::try_from(dst_offset)?;
    let mut remaining = len;

    while remaining > 0 {
        let to_copy = usize::try_from(remaining.min(usize::MAX as u64))?;

        // SAFETY:
        // - `src_fd` and `dst_fd` are valid for the lifetime of the owning
        //   `StdFile` values, which are alive in this call frame.
        // - `off_in = NULL` → kernel reads sequentially from position 0 of source.
        // - `off_out` → explicit write offset; does NOT alter the fd's current
        //   position, avoiding conflicts with O_APPEND semantics.
        // - `flags = 0` (reserved, must be 0).
        let copied = unsafe {
            libc::copy_file_range(
                src_fd,
                std::ptr::null_mut(),
                dst_fd,
                std::ptr::addr_of_mut!(dst_off),
                to_copy,
                0,
            )
        };

        match copied {
            n if n > 0 => remaining -= n as u64,
            0 => bail!("copy_file_range: source exhausted before {len} bytes were copied"),
            _ => {
                let err = std::io::Error::last_os_error();
                if err.raw_os_error() == Some(libc::EINTR) {
                    continue;
                }
                return Err(err.into());
            }
        }
    }

    Ok(len)
}

/// Linux entry point: tries `copy_file_range`, falls back to standard copy.
pub(super) async fn copy_file_to_writer(
    source: &Path,
    dest: &mut File,
    len: u64,
    dest_offset: u64,
) -> Result<u64> {
    let source = source.to_path_buf();

    // Open the source synchronously — we'll move it into `spawn_blocking`.
    let src_std = StdFile::open(&source)?;
    // Clone the destination fd so the kernel-level copy doesn't share state
    // with the async file handle used outside `spawn_blocking`.
    let dest_clone = dest.try_clone().await?.into_std().await;

    // Both `src_std` and `dest_clone` are moved into the closure, ensuring
    // the fds remain valid for the entire duration of the blocking call.
    let result = tokio::task::spawn_blocking(move || {
        try_copy_file_range(&src_std, &dest_clone, len, dest_offset)
        // `src_std` and `dest_clone` are dropped here, closing the fds.
    })
    .await?;

    match result {
        Ok(n) => {
            tracing::debug!(bytes = n, "copy_file_range COW copy succeeded");
            // copy_file_range writes via a cloned fd with an explicit off_out; the
            // original `dest` fd position is not updated by the kernel. Advance it
            // so that subsequent sequential writes land at the correct offset.
            // For O_APPEND handles this seek is harmless (writes always go to EOF).
            dest.seek(std::io::SeekFrom::Start(dest_offset + len))
                .await?;
            Ok(n)
        }
        Err(err) => {
            tracing::debug!(
                error = %err,
                "copy_file_range unavailable, falling back to standard copy"
            );
            super::buffered_copy(&source, dest, len, dest_offset).await
        }
    }
}

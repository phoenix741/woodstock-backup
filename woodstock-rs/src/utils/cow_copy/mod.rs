//! Copy-on-Write file copy utilities.
//!
//! Provides an efficient mechanism to copy a chunk payload from a source file
//! directly into an open segment writer file, using the best copy strategy
//! available for the current OS:
//!
//! - **Linux**: [`copy_file_range(2)`](https://man7.org/linux/man-pages/man2/copy_file_range.2.html)
//!   is used when available. This syscall performs an in-kernel copy that the
//!   filesystem (btrfs, XFS) can satisfy with a COW reference — sharing data
//!   blocks instead of duplicating them. Falls back to a standard read/write
//!   loop if the syscall is not supported (old kernel, cross-device copy, etc.).
//! - **Other platforms**: standard `tokio::io::copy` fallback.

#[cfg(target_os = "linux")]
mod linux;

use std::path::Path;

use eyre::{bail, Result};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

/// Standard buffered copy shared by all platforms.
///
/// Opens `source`, seeks `dest` to `dest_offset`, then streams exactly `len`
/// bytes using `tokio::io::copy` (8 KiB internal buffer).
async fn buffered_copy(source: &Path, dest: &mut File, len: u64, dest_offset: u64) -> Result<u64> {
    let src = File::open(source).await?;

    dest.seek(std::io::SeekFrom::Start(dest_offset)).await?;
    let mut limited = src.take(len);
    let copied = tokio::io::copy(&mut limited, dest).await?;

    if copied != len {
        bail!("copy_file_to_writer: expected {len} bytes, copied {copied}");
    }

    Ok(copied)
}

/// Minimum payload size (bytes) at which `copy_file_range` overhead is
/// justified over a plain buffered copy.
///
/// Benchmarks show the crossover is between 16 KiB (buffered wins) and
/// 64 KiB (`copy_file_range` wins). Set to 64 KiB to be conservative:
///
/// | Size   | copy_file_range | buffered  |
/// |--------|-----------------|-----------|
/// | 16 KiB | 56 µs (279 MiB/s) | 52 µs (301 MiB/s) |
/// | 64 KiB | 70 µs (890 MiB/s) | 110 µs (566 MiB/s) |
#[cfg(target_os = "linux")]
const COW_THRESHOLD: u64 = 64 * 1024; // 64 KiB

/// Copies exactly `len` bytes from the file at `source` into `dest`, starting
/// at `dest_offset` in the destination.
///
/// On Linux, [`copy_file_range(2)`] is attempted first for payloads at or
/// above [`COW_THRESHOLD`] to get in-kernel COW semantics. Below the
/// threshold, or on other platforms, or when the syscall fails (different
/// filesystems, unsupported kernel, etc.), the function falls back silently to
/// a standard buffered copy.
///
/// # Arguments
/// * `source`      – Path of the source file to copy from.
/// * `dest`        – Open destination file. Must be writable; the caller is
///                   responsible for positioning or the function uses an
///                   explicit offset.
/// * `len`         – Exact number of bytes to copy from the beginning of the
///                   source file.
/// * `dest_offset` – Byte offset in the destination file at which to write the
///                   payload (avoids ambiguity with `O_APPEND` handles).
///
/// # Returns
/// The number of bytes actually written (always equal to `len` on success).
///
/// # Errors
/// Returns an error if the source file cannot be opened, if the copy fails
/// after exhausting all fallbacks, or if fewer bytes than `len` are available.
pub async fn copy_file_to_writer(
    source: &Path,
    dest: &mut File,
    len: u64,
    dest_offset: u64,
) -> Result<u64> {
    #[cfg(target_os = "linux")]
    if len >= COW_THRESHOLD {
        return linux::copy_file_to_writer(source, dest, len, dest_offset).await;
    }

    buffered_copy(source, dest, len, dest_offset).await
}

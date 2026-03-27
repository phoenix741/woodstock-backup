//! Default (non-Linux) copy implementation — delegates to the shared buffered copy.

use std::path::Path;

use eyre::Result;
use tokio::fs::File;

/// Copies `len` bytes from `source` into `dest` at `dest_offset` using a
/// standard tokio buffered copy (no COW semantics).
pub(super) async fn copy_file_to_writer(
    source: &Path,
    dest: &mut File,
    len: u64,
    dest_offset: u64,
) -> Result<u64> {
    super::buffered_copy(source, dest, len, dest_offset).await
}

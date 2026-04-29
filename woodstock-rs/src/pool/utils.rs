use std::path::{Path, PathBuf};

use rand::distr::{Alphanumeric, Distribution};

/// # Pool Utilities Module
///
/// This module provides utility functions for chunk path calculation and temporary file management
/// in the Woodstock backup pool. It supports deterministic chunk file placement and secure temporary
/// file creation for atomic operations.
///
/// ## Main Functions
///
/// - [`calculate_chunk_path`]: Compute the path for a chunk file based on its hash.
/// - [`get_temp_chunk_path`]: Generate a secure temporary path for a new chunk file.
///
/// ## Error Handling & Panics
///
/// - Panics if the chunk hash is too short for path calculation (should not happen with valid hashes).
#[must_use]
/// Computes the deterministic path for a chunk file based on its hash.
///
/// # Arguments
///
/// * `pool_path` - The root directory of the chunk pool.
/// * `chunk` - The chunk hash as a string (at least 6 characters).
///
/// # Returns
///
/// The full path to the chunk file in the pool.
///
/// # Panics
///
/// Panics if `chunk` is less than 6 characters.
pub fn calculate_chunk_path(pool_path: &Path, chunk: &str) -> PathBuf {
    let part1 = &chunk[0..2];
    let part2 = &chunk[2..4];
    let part3 = &chunk[4..6];

    Path::new(pool_path)
        .join(part1)
        .join(part2)
        .join(part3)
        .join(chunk.to_string() + "-sha256.zz")
}

/// Generates a secure temporary path for a new chunk file.
///
/// # Arguments
///
/// * `pool_path` - The root directory of the chunk pool.
///
/// # Returns
///
/// The full path to a temporary file in the pool's `_new` directory.
///
/// # Panics
///
/// Panics if random generation fails (very unlikely).
pub fn get_temp_chunk_path(pool_path: &Path) -> PathBuf {
    // Generate a string of 30 characters with random characters (base 36)
    let mut rng = rand::rng();
    let temporary_filename = Alphanumeric
        .sample_iter(&mut rng)
        .take(30)
        .map(char::from)
        .collect::<String>();

    Path::new(pool_path).join("_new").join(temporary_filename)
}

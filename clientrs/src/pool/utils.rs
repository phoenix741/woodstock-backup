use std::path::{Path, PathBuf};

use rand::Rng;

#[must_use] pub fn calculate_chunk_path(pool_path: &Path, chunk: &str) -> PathBuf {
    let part1 = &chunk[0..2];
    let part2 = &chunk[2..4];
    let part3 = &chunk[4..6];

    Path::new(pool_path)
        .join(part1)
        .join(part2)
        .join(part3)
        .join(chunk.to_string() + "-sha256.zz")
}

pub fn get_temp_chunk_path(pool_path: &Path) -> PathBuf {
    // Generate a string of 30 characters with random characters (base 36)
    let temporary_filename = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(30)
        .map(char::from)
        .collect::<String>();

    Path::new(pool_path).join("_new").join(temporary_filename)
}

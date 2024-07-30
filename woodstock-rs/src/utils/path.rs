use globset::{GlobBuilder, GlobSetBuilder};
use percent_encoding::{percent_decode_str, utf8_percent_encode, NON_ALPHANUMERIC};
use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

/// Converts a vector of byte vectors to a vector of string slices.
///   
/// # Arguments
///
/// * `vec` - A vector of byte vectors.
///
/// # Returns
///
/// A vector of string slices.
///
#[must_use]
pub fn vec_to_str(vec: &Vec<String>) -> Vec<&str> {
    let mut vec_of_str: Vec<&str> = Vec::new();

    for value in vec {
        vec_of_str.push(value);
    }

    vec_of_str
}

/// Converts a path to a vector of bytes.
///
/// # Arguments
///
/// * `path` - A path.
///
/// # Returns
///
/// A vector of bytes.
///
#[must_use]
pub fn osstr_to_vec(path: &OsStr) -> Vec<u8> {
    path.as_encoded_bytes().to_vec()
}

/// Converts a vector of bytes to a `PathBuf`.
///
/// # Arguments
///
/// * `vec` - A vector of bytes.
///
/// # Returns
///
/// A `PathBuf`.
///
#[must_use]
pub fn vec_to_osstr(vec: &[u8]) -> OsString {
    unsafe { OsString::from_encoded_bytes_unchecked(vec.to_owned()) }
}

/// Converts a path to a vector of bytes.
///
/// # Arguments
///
/// * `path` - A path.
///
/// # Returns
///
/// A vector of bytes.
///
#[must_use]
pub fn path_to_vec(path: &Path) -> Vec<u8> {
    osstr_to_vec(path.as_os_str())
}

/// Converts a vector of bytes to a `PathBuf`.
///
/// # Arguments
///
/// * `vec` - A vector of bytes.
///
/// # Returns
///
/// A `PathBuf`.
///
#[must_use]
pub fn vec_to_path(vec: &[u8]) -> PathBuf {
    PathBuf::from(vec_to_osstr(vec))
}

/// Converts a list of string slices to a `GlobSet`.
///
/// # Arguments
///
/// * `list` - A list of string slices.
///
/// # Returns
///
/// A `Result` containing the `GlobSet` if successful, or an error if the pattern cannot be parsed.
///
/// # Errors
///
/// An error is returned if the pattern cannot be parsed.
///
pub fn list_to_globset(list: &[&str]) -> Result<globset::GlobSet, globset::Error> {
    let mut builder = GlobSetBuilder::new();
    for pattern in list {
        builder.add(GlobBuilder::new(pattern).build()?);
    }
    builder.build()
}

/// Take a path and mangle it by replacing special characters like the
/// method encodeURIComponent will do in javascript.
///
/// # Arguments
///
/// * `path` - A string slice representing the path to mangle.
///
/// # Returns
///
/// A string slice representing the mangled path.
///
#[must_use]
pub fn mangle(path: &str) -> String {
    utf8_percent_encode(path, NON_ALPHANUMERIC).to_string()
}

/// Take a mangled path and unmangle it by decoding percent-encoded characters.
///
/// # Arguments
///
/// * `path` - A string slice representing the mangled path.
///
/// # Returns
///
/// A string slice representing the unmangled path.
///
#[must_use]
pub fn unmangle(path: &str) -> String {
    percent_decode_str(path).decode_utf8_lossy().to_string()
}

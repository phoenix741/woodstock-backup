use globset::{GlobBuilder, GlobSetBuilder};
use percent_encoding::{
    percent_decode, percent_decode_str, percent_encode, utf8_percent_encode, NON_ALPHANUMERIC,
};
use std::{
    collections::HashSet,
    ffi::{OsStr, OsString},
    hash::Hash,
    path::{Component, Path, PathBuf, MAIN_SEPARATOR_STR},
};
use tracing::warn;

/// Converts a vector of byte vectors to a vector of string slices.
///
/// # Arguments
/// * `vec` - A vector of byte vectors.
///
/// # Returns
///
/// A vector of string slices.
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
/// * `path` - A path.
///
/// # Returns
///
/// A vector of bytes.
#[must_use]
pub fn osstr_to_vec(path: &OsStr) -> Vec<u8> {
    path.as_encoded_bytes().to_vec()
}

/// Converts a string to a vector of bytes.
#[must_use]
pub fn str_to_vec(path: &str) -> Vec<u8> {
    path.as_bytes().to_vec()
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
pub fn path_to_vec<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let path = path.as_ref();
    let components = path.components();
    let mut buff = Vec::new();
    for component in components {
        match component {
            Component::Normal(path) => {
                buff.extend(osstr_to_vec(path));
                buff.push(b'/');
            }
            Component::RootDir => {
                buff.push(b'/');
            }
            Component::Prefix(prefix) => {
                buff.extend(osstr_to_vec(prefix.as_os_str()));
            }
            _ => {
                warn!("Unsupported path component: {:?}", component);
            }
        }
    }
    if buff.len() > 1 {
        buff.pop();
    }
    buff
}

/// Converts a vector of bytes to a `PathBuf`.
///
/// # Arguments
/// * `vec` - A vector of bytes.
///
/// # Returns
///
/// A `PathBuf` representing the path.
#[must_use]
pub fn vec_to_path(vec: &[u8]) -> PathBuf {
    // Create a new string, replace all b'/' and b'\\' with MAIN_SEPARTOR
    let vec = vec
        .iter()
        .map(|&byte| {
            if byte == b'/' || byte == b'\\' {
                MAIN_SEPARATOR_STR.as_bytes()[0]
            } else {
                byte
            }
        })
        .collect::<Vec<u8>>();
    let osstr = vec_to_osstr(&vec);
    PathBuf::from(osstr)
}

/// Converts a list of string slices to a `GlobSet`.
///
/// # Arguments
/// * `list` - A list of string slices.
///
/// # Returns
///
/// * `Ok(GlobSet)` if the conversion is successful.
/// * `Err(globset::Error)` if an error occurs during conversion.
///
/// # Errors
///
/// Returns an error if the glob pattern cannot be parsed.
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

/// Take a buffer mangle it by replacing special characters like the
/// method encodeURIComponent will do in javascript.
///
/// # Arguments
///
/// * `path` - A string slice representing the path to mangle.

/// # Returns
///
/// A string slice representing the mangled path.
#[must_use]
pub fn mangle_buffer(bytes: &[u8]) -> String {
    percent_encode(&bytes, NON_ALPHANUMERIC).to_string()
}

/// Take a path and mangle it by replacing special characters like the
/// method encodeURIComponent will do in javascript.
///
/// # Arguments
///
/// * `path` - A string slice representing the path to mangle.

/// # Returns
///
/// A string slice representing the mangled path.
#[must_use]
pub fn mangle_path<P: AsRef<Path>>(path: P) -> String {
    let bytes = path_to_vec(path);
    percent_encode(&bytes, NON_ALPHANUMERIC).to_string()
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

/// Take a mangled string unmangle it by decoding percent-encoded characters.
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
pub fn unmangle_buffer(path: &str) -> Vec<u8> {
    percent_decode(path.as_bytes()).collect::<Vec<u8>>()
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
pub fn unmangle_path(path: &str) -> PathBuf {
    let decoded = percent_decode(path.as_bytes()).collect::<Vec<u8>>();
    vec_to_path(&decoded)
}

/// Filter all value to return only unique values
///
/// # Arguments
///
/// * `iterable` - The iterable to filter
///
/// # Returns
///
/// A new iterable with only unique values
#[must_use]
pub fn unique<T: Eq + Hash + Clone>(iterable: impl IntoIterator<Item = T>) -> Vec<T> {
    let unique_elts: HashSet<T> = HashSet::from_iter(iterable);
    unique_elts.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    // Test vec_to_path and path_to_vec
    #[test]
    fn test_path_conversion() {
        let path = Path::new("/test/path/to/convert");
        let vec = super::path_to_vec(path);
        let new_path = super::vec_to_path(&vec);
        assert_eq!(new_path, Path::new("/test/path/to/convert"));
    }

    // Test vec_to_path and path_to_vec
    #[test]
    #[cfg(windows)]
    fn test_path_conversion_windows() {
        let path = Path::new("\\test\\path\\to\\convert");
        let vec = super::path_to_vec(path);
        let new_path = super::vec_to_path(&vec);
        println!("{:?}", new_path);
        assert_eq!(new_path, Path::new("\\test\\\\path\\to\\convert"));

        let path = Path::new("C:\\test\\path\\to\\convert");
        let vec = super::path_to_vec(path);
        let new_path = super::vec_to_path(&vec);
        assert_eq!(new_path, Path::new("C:\\test\\path\\to\\convert"));

        let path = Path::new("C:/Tools/a/b/c");
        let vec = super::path_to_vec(path);
        let new_path = super::vec_to_path(&vec);
        assert_eq!(new_path, Path::new("C:\\Tools\\a\\b\\c"));

        let vec = vec![67, 58, 92, 84, 111, 111, 108, 115, 47, 97, 47, 98, 47, 99];
        let new_path = super::vec_to_path(&vec);
        println!("{:?}", new_path);
        assert_eq!(new_path, Path::new("C:\\Tools\\a\\b\\c"));
    }
}

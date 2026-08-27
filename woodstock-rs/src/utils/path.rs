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
/// A pattern with a trailing `/` (e.g. `"**/vendor/"`, used to mark "this
/// must be a directory" in rsync-style exclude lists) is stripped of that
/// slash before compiling: matched paths never carry a trailing separator
/// (see `client-rs`'s scanner, which joins child names without appending
/// one), so a pattern ending in `/` would otherwise compile to a glob that
/// can never match anything — a silently dead exclude/include rule.
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
        let pattern = pattern.trim_end_matches('/');
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

/// Maps a raw share identifier — a POSIX path like `/etc`, or a Windows
/// share like `C:\` or `C:\Users` — to a safe relative path prefix, usable
/// both as a tar archive entry path and as a filesystem directory name.
///
/// Windows shares are stored verbatim (e.g. `C:\`, see `hosts/<name>.yml`).
/// Joining that string directly onto another path is unsafe: on the Linux
/// server, `\` is not a path separator, so the whole share collapses into a
/// single opaque component containing a literal backslash and colon —
/// `tar`'s stricter path validation (e.g. `Header::set_link_name`) rejects
/// it outright, and on plain disk it creates one oddly-named directory
/// instead of a `C/Users/...` tree.
///
/// Backslashes become forward slashes, the drive letter's `:` is dropped,
/// and empty/leading-separator components are discarded. `.` and `..`
/// components are also discarded, so a malformed `share` value (e.g. from
/// a hand-edited `hosts.yml`) can never walk the result outside the root
/// it gets `.join()`ed onto.
///
/// # Arguments
///
/// * `share` - The raw share identifier to normalize.
///
/// # Returns
///
/// A relative [`PathBuf`] safe to `.join()` onto an archive or destination
/// root. Never absolute, never empty for a non-empty `share`.
#[must_use]
pub fn safe_share_prefix(share: &str) -> PathBuf {
    let normalized = share.replace('\\', "/").replace(':', "");
    normalized
        .split('/')
        .filter(|c| !c.is_empty() && *c != "." && *c != "..")
        .collect()
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

    #[test]
    fn test_safe_share_prefix_windows_drive_root() {
        assert_eq!(super::safe_share_prefix("C:\\"), Path::new("C"));
    }

    #[test]
    fn test_safe_share_prefix_windows_subpath() {
        assert_eq!(
            super::safe_share_prefix("C:\\Users\\evero"),
            Path::new("C/Users/evero")
        );
    }

    #[test]
    fn test_safe_share_prefix_posix_root_share() {
        assert_eq!(super::safe_share_prefix("/etc"), Path::new("etc"));
    }

    #[test]
    fn test_safe_share_prefix_posix_nested_share() {
        assert_eq!(
            super::safe_share_prefix("/srv/my-data"),
            Path::new("srv/my-data")
        );
    }

    #[test]
    fn test_safe_share_prefix_strips_parent_dir_traversal() {
        assert_eq!(
            super::safe_share_prefix("/srv/../../etc"),
            Path::new("srv/etc")
        );
    }

    #[test]
    fn test_safe_share_prefix_strips_leading_traversal() {
        assert_eq!(
            super::safe_share_prefix("../../etc/passwd"),
            Path::new("etc/passwd")
        );
    }

    #[test]
    fn test_safe_share_prefix_strips_current_dir() {
        assert_eq!(
            super::safe_share_prefix("/srv/./data"),
            Path::new("srv/data")
        );
    }

    // A pattern ending in '/' (e.g. copied from an rsync exclude list to mean
    // "directory only") must still match, even though matched paths never
    // carry a trailing separator. Without stripping it, the pattern is a
    // silently dead no-op — see `list_to_globset`.
    #[test]
    fn test_list_to_globset_matches_recursive_pattern_with_trailing_slash() {
        let globset = super::list_to_globset(&["**/vendor/"]).unwrap();
        assert!(globset.is_match("project/vendor"));
        assert!(globset.is_match("vendor"));
        assert!(!globset.is_match("project/vendor-extra"));
    }

    #[test]
    fn test_list_to_globset_matches_literal_pattern_with_trailing_slash() {
        let globset = super::list_to_globset(&[".cache/lm-studio/"]).unwrap();
        assert!(globset.is_match(".cache/lm-studio"));
        assert!(!globset.is_match("other/.cache/lm-studio"));
    }

    #[test]
    fn test_list_to_globset_still_matches_pattern_without_trailing_slash() {
        let globset = super::list_to_globset(&["**/node_modules"]).unwrap();
        assert!(globset.is_match("project/node_modules"));
    }
}

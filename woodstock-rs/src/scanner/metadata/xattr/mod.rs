#[cfg(all(unix, feature = "xattr"))]
mod unix;
#[cfg(not(all(unix, feature = "xattr")))]
mod windows;

#[cfg(all(unix, feature = "xattr"))]
pub use unix::*;

#[cfg(not(all(unix, feature = "xattr")))]
pub use windows::*;

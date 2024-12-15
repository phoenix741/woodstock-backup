#[cfg(all(unix, feature = "acl"))]
mod unix;
#[cfg(not(all(unix, feature = "acl")))]
mod windows;

#[cfg(all(unix, feature = "acl"))]
pub use unix::*;

#[cfg(not(all(unix, feature = "acl")))]
pub use windows::*;

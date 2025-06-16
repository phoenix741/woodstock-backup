//! Abort handle utility for Woodstock server operations.
//!
//! This module provides the `AbortHandle` struct, which allows for the cancellation of asynchronous tasks
//! such as backup or restore operations in the Woodstock backup system.

use tokio::task::JoinHandle;

#[napi(js_name = "AbortHandle")]
/// Handle for aborting asynchronous tasks.
pub struct AbortHandle {
  /// Tokio join handle for the async task.
  handle: JoinHandle<()>,
}

#[napi]
impl AbortHandle {
  #[must_use]
  pub fn new(handle: JoinHandle<()>) -> Self {
    Self { handle }
  }

  #[napi]
  pub fn abort(&self) {
    self.handle.abort();
  }
}

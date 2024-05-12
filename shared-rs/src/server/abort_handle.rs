use tokio::task::JoinHandle;

#[napi(js_name = "AbortHandle")]
pub struct AbortHandle {
  handle: JoinHandle<()>,
}

#[napi]
impl AbortHandle {
  pub fn new(handle: JoinHandle<()>) -> Self {
    Self { handle }
  }

  #[napi]
  pub async fn abort(&self) {
    self.handle.abort();
  }
}

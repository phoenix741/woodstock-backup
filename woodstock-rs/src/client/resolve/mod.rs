mod mdns;
pub use mdns::MdnsResolveClient;
mod direct;
pub use direct::DirectResolveClient;

use eyre::Result;

#[tonic::async_trait]
pub trait ResolveClient {
    async fn start(&self) -> Result<()>;
    async fn stop(&self);
    async fn shutdown(&self);
}

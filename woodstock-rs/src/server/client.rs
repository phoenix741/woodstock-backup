use futures::Stream;

use crate::AuthenticateReply;
use crate::ChunkHashReply;
use crate::ChunkHashRequest;
use crate::ChunkInformation;
use crate::ExecuteCommandReply;
use crate::FileChunk;
use crate::FileManifestJournalEntry;
use crate::RefreshCacheRequest;
use eyre::Result;

#[tonic::async_trait]
pub trait Client {
    async fn ping(&self) -> Result<bool>;

    async fn authenticate(&mut self, password: &str) -> Result<AuthenticateReply>;

    async fn execute_command(&mut self, command: &str) -> Result<ExecuteCommandReply>;

    fn synchronize_file_list(
        &mut self,
        cache: impl Stream<Item = RefreshCacheRequest> + Send + Sync + 'static,
    ) -> impl Stream<Item = Result<FileManifestJournalEntry>> + '_;

    async fn get_chunk_hash(&self, request: ChunkHashRequest) -> Result<ChunkHashReply>;

    fn get_chunk(&self, request: ChunkInformation) -> impl Stream<Item = Result<FileChunk>> + '_;

    async fn close(&self) -> Result<()>;
}

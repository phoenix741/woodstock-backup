use futures::Stream;

use crate::ChunkHashReply;
use crate::ChunkHashRequest;
use crate::ChunkInformation;
use crate::ExecuteCommandReply;
use crate::FileChunk;
use crate::FileManifestJournalEntry;
use crate::LaunchBackupRequest;
use crate::RefreshCacheRequest;
use crate::{AuthenticateReply, Empty as ProtoEmpty};
use eyre::Result;

#[tonic::async_trait]
pub trait Client {
    async fn authenticate(&mut self, password: &str) -> Result<AuthenticateReply>;

    async fn execute_command(&mut self, command: &str) -> Result<ExecuteCommandReply>;

    async fn refresh_cache(
        &mut self,
        cache: impl Stream<Item = RefreshCacheRequest> + Send + Sync + 'static,
    ) -> Result<ProtoEmpty>;

    fn download_file_list(
        &mut self,
        request: LaunchBackupRequest,
    ) -> impl Stream<Item = Result<FileManifestJournalEntry>> + '_;

    async fn get_chunk_hash(&self, request: ChunkHashRequest) -> Result<ChunkHashReply>;

    fn get_chunk(&self, request: ChunkInformation) -> impl Stream<Item = Result<FileChunk>> + '_;

    async fn close(&self) -> Result<()>;
}

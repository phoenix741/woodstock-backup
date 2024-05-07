use futures::Stream;

use crate::ChunkInformation;
use crate::ExecuteCommandReply;
use crate::FileChunk;
use crate::FileManifestJournalEntry;
use crate::LaunchBackupRequest;
use crate::RefreshCacheRequest;
use crate::{AuthenticateReply, Empty as ProtoEmpty, LogEntry};

#[tonic::async_trait]
pub trait Client {
    async fn authenticate(
        &mut self,
        password: &str,
    ) -> Result<AuthenticateReply, Box<dyn std::error::Error>>;

    fn stream_log(
        &mut self,
    ) -> impl Stream<Item = Result<LogEntry, Box<dyn std::error::Error + Send + Sync>>> + '_;

    async fn execute_command(
        &mut self,
        command: &str,
    ) -> Result<ExecuteCommandReply, Box<dyn std::error::Error>>;

    async fn refresh_cache(
        &mut self,
        cache: impl Stream<Item = RefreshCacheRequest> + Send + Sync + 'static,
    ) -> Result<ProtoEmpty, Box<dyn std::error::Error>>;

    fn download_file_list(
        &mut self,
        request: LaunchBackupRequest,
    ) -> impl Stream<Item = Result<FileManifestJournalEntry, Box<dyn std::error::Error + Send + Sync>>>
           + '_;

    fn get_chunk(
        &self,
        request: ChunkInformation,
    ) -> impl Stream<Item = Result<FileChunk, Box<dyn std::error::Error + Send + Sync>>> + '_;

    async fn close(&self) -> Result<(), Box<dyn std::error::Error>>;
}

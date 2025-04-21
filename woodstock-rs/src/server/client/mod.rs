/// This module defines the grpc client
pub mod grpc;

use futures::Stream;

use crate::AuthenticateReply;
use crate::ChunkHashReply;
use crate::ChunkHashRequest;
use crate::ChunkInformation;
use crate::ExecuteCommandReply;
use crate::FileChunk;
use crate::FileManifestJournalEntry;
use crate::RefreshCacheRequest;
use crate::RestoreFileReply;
use crate::RestoreFileRequest;
use eyre::Result;

#[tonic::async_trait]
pub trait Client {
    /// Sends a ping request to the server to check connectivity.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if the server responds to the ping.
    /// * `Ok(false)` if the server does not respond.
    /// * `Err(eyre::Report)` if an error occurs during the ping.
    async fn ping(&self) -> Result<bool>;

    /// Authenticates the client with the server using the provided password.
    ///
    /// # Arguments
    /// * `password` - The password for authentication.
    ///
    /// # Returns
    ///
    /// * `Ok(AuthenticateReply)` if authentication is successful.
    /// * `Err(eyre::Report)` if an error occurs during authentication.
    async fn authenticate(&self, password: &str) -> Result<AuthenticateReply>;

    /// Executes a command on the server.
    ///
    /// # Arguments
    /// * `command` - The command to execute.
    ///
    /// # Returns
    ///
    /// * `Ok(ExecuteCommandReply)` if the command execution is successful.
    /// * `Err(eyre::Report)` if an error occurs during command execution.
    async fn execute_command(&self, command: &str) -> Result<ExecuteCommandReply>;

    /// Synchronizes the file list with the server.
    ///
    /// # Arguments
    /// * `cache` - A stream of `RefreshCacheRequest` items to send to the server.
    ///
    /// # Returns
    ///
    /// A stream of `Result<FileManifestJournalEntry>` items representing the synchronized file list.
    fn synchronize_file_list(
        &self,
        cache: impl Stream<Item = RefreshCacheRequest> + Send + Sync + 'static,
    ) -> impl Stream<Item = Result<FileManifestJournalEntry>> + '_;

    /// Restores files from the server.
    ///
    /// # Arguments
    /// * `requests` - A stream of `RestoreFileRequest` items to send to the server.
    ///
    /// # Returns
    ///
    /// A stream of `Result<RestoreFileReply>` items representing the restored files.
    fn restore_file(
        &self,
        requests: impl Stream<Item = RestoreFileRequest> + Send + Sync + 'static,
    ) -> impl Stream<Item = Result<RestoreFileReply>> + '_;

    /// Retrieves the hash of a chunk from the server.
    ///
    /// # Arguments
    /// * `request` - The `ChunkHashRequest` containing the chunk information.
    ///
    /// # Returns
    ///
    /// * `Ok(ChunkHashReply)` if the hash retrieval is successful.
    /// * `Err(eyre::Report)` if an error occurs during hash retrieval.
    async fn get_chunk_hash(&self, request: ChunkHashRequest) -> Result<ChunkHashReply>;

    /// Retrieves a chunk from the server.
    ///
    /// # Arguments
    /// * `request` - The `ChunkInformation` containing the chunk details.
    ///
    /// # Returns
    ///
    /// A stream of `Result<FileChunk>` items representing the retrieved chunk.
    fn get_chunk(&self, request: ChunkInformation) -> impl Stream<Item = Result<FileChunk>> + '_;

    /// Closes the client connection to the server.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the connection is successfully closed.
    /// * `Err(eyre::Report)` if an error occurs during closure.
    async fn close(&self) -> Result<()>;
}

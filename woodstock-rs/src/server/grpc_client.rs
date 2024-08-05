use async_stream::try_stream;
use eyre::eyre;
use futures::pin_mut;
use futures::Stream;
use log::{error, info};
use std::path::Path;
use std::path::PathBuf;
use tokio::fs::read_to_string;
use tonic::{
    transport::{Certificate, Channel, ClientTlsConfig, Identity},
    Request,
};

use super::client::Client;
use crate::config::Context;
use crate::woodstock;
use crate::ChunkHashReply;
use crate::ChunkHashRequest;
use crate::ChunkInformation;
use crate::ExecuteCommandReply;
use crate::ExecuteCommandRequest;
use crate::FileChunk;
use crate::FileManifestJournalEntry;
use crate::LaunchBackupRequest;
use crate::RefreshCacheRequest;
use crate::{
    utils::encryption::create_authentification_token,
    woodstock_client_service_client::WoodstockClientServiceClient, AuthenticateReply,
    AuthenticateRequest, Empty as ProtoEmpty,
};
use eyre::Result;

#[derive(Clone)]
pub struct BackupGrpcClient {
    hostname: String,

    certs_path: PathBuf,

    client: WoodstockClientServiceClient<Channel>,
    session_id: Option<String>,
}

impl BackupGrpcClient {
    fn create_request<T>(&self, request: T) -> std::result::Result<Request<T>, tonic::Status> {
        let session_id = self.session_id.clone();

        if let Some(session_id) = session_id {
            let mut request = Request::new(request);
            request.metadata_mut().insert(
                "x-session-id",
                session_id
                    .parse()
                    .map_err(|_| tonic::Status::invalid_argument("Can't parse x-session-id"))?,
            );

            Ok(request)
        } else {
            error!("No session id available to stream logs");
            Err(tonic::Status::failed_precondition(
                "No session id available",
            ))
        }
    }

    async fn create_client(
        hostname: &str,
        ip: &str,
        certs_path: &PathBuf,
    ) -> Result<WoodstockClientServiceClient<Channel>> {
        info!("Connecting to {hostname}, {ip}");
        if ip.is_empty() {
            error!("No IP address provided for {hostname}");
            return Err(eyre!("No IP address provided"));
        }

        let certificate_path = &certs_path;
        let server_root_ca_cert =
            read_to_string(certificate_path.join(format!("{hostname}_ca.pem"))).await?;
        let server_root_ca_cert = Certificate::from_pem(server_root_ca_cert);
        let client_cert =
            read_to_string(certificate_path.join(format!("{hostname}_client.pem"))).await?;
        let client_key =
            read_to_string(certificate_path.join(format!("{hostname}_client.key"))).await?;
        let client_identity = Identity::from_pem(client_cert, client_key);

        let tls = ClientTlsConfig::new()
            .domain_name(hostname)
            .ca_certificate(server_root_ca_cert)
            .identity(client_identity);

        let connection_string = format!("https://{ip}:3657");
        let channel = Channel::from_shared(connection_string)?
            .tls_config(tls)?
            .connect()
            .await?;

        Ok(WoodstockClientServiceClient::new(channel))
    }

    pub async fn new(hostname: &str, ip: &str, ctxt: &Context) -> Result<Self> {
        info!("Creating BackupGrpcClient with hostname = {hostname}, ip = {ip}");
        let certs_path = ctxt.config.path.certificates_path.clone();
        let client = BackupGrpcClient::create_client(hostname, ip, &certs_path).await?;

        Ok(BackupGrpcClient {
            hostname: hostname.to_string(),
            certs_path,

            client,
            session_id: None,
        })
    }

    pub fn with_client(
        hostname: &str,
        ip: &str,
        client: WoodstockClientServiceClient<Channel>,
        certs_path: &Path,
    ) -> Self {
        info!("Creating BackupGrpcClient with hostname = {hostname}, ip = {ip}");
        BackupGrpcClient {
            hostname: hostname.to_string(),
            certs_path: certs_path.to_path_buf(),

            client,
            session_id: None,
        }
    }
}

#[tonic::async_trait]
impl Client for BackupGrpcClient {
    async fn authenticate(&mut self, password: &str) -> Result<AuthenticateReply> {
        let mut client = self.client.clone();

        let token =
            create_authentification_token(&self.certs_path, &self.hostname, password).await?;

        let response = client
            .authenticate(AuthenticateRequest { token, version: 0 })
            .await?;

        let session_id = &response.get_ref().session_id;
        self.session_id = Some(session_id.clone());

        Ok(response.into_inner())
    }

    async fn execute_command(&mut self, command: &str) -> Result<ExecuteCommandReply> {
        let mut client = self.client.clone();

        let request = self.create_request(ExecuteCommandRequest {
            command: command.to_string(),
        })?;

        let response = client.execute_command(request).await?;

        Ok(response.into_inner())
    }

    async fn refresh_cache(
        &mut self,
        cache: impl Stream<Item = RefreshCacheRequest> + Send + Sync + 'static,
    ) -> Result<ProtoEmpty> {
        let mut client = self.client.clone();
        let request = self.create_request(cache)?;
        let reply = client.refresh_cache(request).await?;

        Ok(reply.into_inner())
    }

    fn download_file_list(
        &mut self,
        request: LaunchBackupRequest,
    ) -> impl Stream<Item = Result<FileManifestJournalEntry>> + '_ {
        let client = self.client.clone();

        try_stream!({
            let mut client = client;

            let request = self.create_request(request)?;

            let messages = client.launch_backup(request).await?;
            let mut messages = messages.into_inner();

            while let Some(message) = messages.message().await? {
                yield message;
            }
        })
    }

    async fn get_chunk_hash(&self, request: ChunkHashRequest) -> Result<ChunkHashReply> {
        let client = self.client.clone();
        let mut client = client;

        let request = self.create_request(request)?;

        let result = client.get_chunk_hash(request).await?;

        Ok(result.into_inner())
    }

    fn get_chunk(&self, request: ChunkInformation) -> impl Stream<Item = Result<FileChunk>> + '_ {
        try_stream!({
            let client = self.client.clone();
            pin_mut!(client);

            let request = self.create_request(request)?;
            let chunks = client.get_chunk(request).await?.into_inner();
            pin_mut!(chunks);

            while let Some(chunks) = chunks.message().await? {
                yield chunks;
            }
        })
    }

    async fn close(&self) -> Result<()> {
        let client = self.client.clone();
        let mut client = client;

        let request = self.create_request(woodstock::Empty {})?;

        client.close_backup(request).await?;

        Ok(())
    }
}

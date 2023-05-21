use async_stream::try_stream;
use futures::pin_mut;
use futures::Stream;
use log::{error, info};
use std::path::PathBuf;
use tonic::{
    transport::{Certificate, Channel, ClientTlsConfig, Identity},
    Request,
};

use crate::woodstock;
use crate::ChunkInformation;
use crate::ExecuteCommandReply;
use crate::ExecuteCommandRequest;
use crate::FileChunk;
use crate::FileManifestJournalEntry;
use crate::LaunchBackupRequest;
use crate::RefreshCacheRequest;
use crate::{
    authentification::encryption::create_authentification_token, config::ConfigurationPath,
    woodstock_client_service_client::WoodstockClientServiceClient, AuthenticateReply,
    AuthenticateRequest, Empty as ProtoEmpty, LogEntry, StreamLogRequest,
};

#[derive(Clone)]
pub struct BackupGrpcClient {
    hostname: String,

    certs_path: PathBuf,

    client: WoodstockClientServiceClient<Channel>,
    session_id: Option<String>,
}

impl BackupGrpcClient {
    fn create_request<T>(&self, request: T) -> Result<Request<T>, tonic::Status> {
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
    ) -> Result<WoodstockClientServiceClient<Channel>, Box<dyn std::error::Error>> {
        info!("Connecting to {hostname}, {ip}");
        if ip.is_empty() {
            error!("No IP address provided for {hostname}");
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No IP address provided",
            )));
        }

        let certificate_path = &certs_path;
        let server_root_ca_cert =
            std::fs::read_to_string(certificate_path.join(format!("{hostname}_ca.pem")))?;
        let server_root_ca_cert = Certificate::from_pem(server_root_ca_cert);
        let client_cert =
            std::fs::read_to_string(certificate_path.join(format!("{hostname}_client.pem")))?;
        let client_key =
            std::fs::read_to_string(certificate_path.join(format!("{hostname}_client.key")))?;
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

    pub async fn new(hostname: &str, ip: &str) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Creating BackupGrpcClient with hostname = {hostname}, ip = {ip}");
        let certs_path = ConfigurationPath::default().certificates_path;
        let client = BackupGrpcClient::create_client(hostname, ip, &certs_path).await?;

        Ok(BackupGrpcClient {
            hostname: hostname.to_string(),
            certs_path,

            client,
            session_id: None,
        })
    }

    pub async fn authenticate(
        &mut self,
        password: &str,
    ) -> Result<AuthenticateReply, Box<dyn std::error::Error>> {
        let mut client = self.client.clone();

        let token = create_authentification_token(&self.certs_path, &self.hostname, password)?;

        let response = client
            .authenticate(AuthenticateRequest { token, version: 0 })
            .await?;

        let session_id = &response.get_ref().session_id;
        self.session_id = Some(session_id.clone());

        Ok(response.into_inner())
    }

    pub fn stream_log(
        &mut self,
    ) -> impl Stream<Item = Result<LogEntry, Box<dyn std::error::Error + Send + Sync>>> + '_ {
        try_stream!({
            let mut client = self.client.clone();

            let request = self.create_request(StreamLogRequest {})?;

            let logs = client.stream_log(request).await?;
            let mut logs = logs.into_inner();

            while let Some(log) = logs.message().await? {
                yield log;
            }
        })
    }

    pub async fn execute_command(
        &mut self,
        command: &str,
    ) -> Result<ExecuteCommandReply, Box<dyn std::error::Error>> {
        let mut client = self.client.clone();

        let request = self.create_request(ExecuteCommandRequest {
            command: command.to_string(),
        })?;

        let response = client.execute_command(request).await?;

        Ok(response.into_inner())
    }

    pub async fn refresh_cache(
        &mut self,
        cache: impl Stream<Item = RefreshCacheRequest> + Send + Sync + 'static,
    ) -> Result<ProtoEmpty, Box<dyn std::error::Error>> {
        let mut client = self.client.clone();
        let request = self.create_request(cache)?;
        let reply = client.refresh_cache(request).await?;

        Ok(reply.into_inner())
    }

    pub fn download_file_list(
        &mut self,
        request: LaunchBackupRequest,
    ) -> impl Stream<Item = Result<FileManifestJournalEntry, Box<dyn std::error::Error + Send + Sync>>>
           + '_ {
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

    pub fn get_chunk(
        &self,
        request: ChunkInformation,
    ) -> impl Stream<Item = Result<FileChunk, Box<dyn std::error::Error + Send + Sync>>> + '_ {
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

    pub async fn close(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.client.clone();
        let mut client = client;

        let request = self.create_request(woodstock::Empty {})?;

        let result = client.close_backup(request).await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }
}

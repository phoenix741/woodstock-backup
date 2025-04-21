use eyre::Result;
use futures::{pin_mut, StreamExt};
use log::{error, info};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};
use tokio::{
    io::AsyncReadExt,
    sync::{mpsc, Mutex},
};
use uuid::Uuid;

use super::super::client::Client;
use crate::{
    config::{Backups, Configuration, Context, BUFFER_SIZE, DEFAULT_CHANNEL_BUFFER_SIZE},
    event::Information,
    events::append_events,
    restore_file_request,
    server::progression::BackupProgression,
    utils::path::{path_to_vec, vec_to_path},
    Event, EventBackupInformation, EventSource, EventStatus, EventStep, EventType,
    RestoreFileRequest,
};

pub struct BackupRestore<Clt: Client> {
    /// The unique identifier for the restore operation.
    uuid: Vec<u8>,
    /// The client responsible for the restore operation.
    client: Clt,
    /// The hostname associated with the restore operation.
    hostname: String,
    /// The ID of the current backup being restored.
    current_backup_id: usize,
    /// A map tracking the maximum progress for each task.
    progress_max: Arc<Mutex<HashMap<String, u64>>>,
    /// The progression state of the restore operation.
    progression: Arc<Mutex<BackupProgression>>,
    /// The source of events for the restore operation.
    source: EventSource,
    /// The configuration for the restore operation.
    config: Configuration,
    /// The path to the storage pool.
    pool_path: PathBuf,
}

impl<Clt: Client> BackupRestore<Clt> {
    /// Creates a new instance of `BackupRestore`.
    ///
    /// # Arguments
    /// * `client` - The client used for restoration.
    /// * `hostname` - The hostname of the backup to restore.
    /// * `backup_number` - The backup number to restore.
    /// * `ctxt` - The context containing the event source.
    /// * `config` - The configuration for the backup system.
    ///
    /// # Returns
    ///
    /// A new instance of `BackupRestore`.
    pub fn new(
        client: Clt,
        hostname: &str,
        backup_number: usize,
        ctxt: &Context,
        config: &Configuration,
    ) -> Self {
        let backups = Backups::new(config);
        let destination_directory =
            backups.get_backup_destination_directory(hostname, backup_number);

        info!(
            "Initialize backup client for restauration {hostname}/{backup_number} in {destination_directory:?}"
        );
        let uuid = Uuid::new_v4();
        let uuid = uuid.as_bytes().to_vec();

        BackupRestore {
            uuid,
            client,
            hostname: hostname.to_string(),
            current_backup_id: backup_number,
            progress_max: Arc::new(Mutex::new(HashMap::new())),
            progression: Arc::new(Mutex::new(BackupProgression::default())),
            source: ctxt.source,
            config: config.clone(),
            pool_path: config.path.pool_path.clone(),
        }
    }

    /// Retrieves the current progression state.
    ///
    /// # Returns
    ///
    /// The current `BackupProgression` state.
    pub async fn progress(&self) -> BackupProgression {
        *self.progression.lock().await
    }

    /// Authenticates the client with the provided password.
    ///
    /// # Arguments
    /// * `password` - The password for authentication.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the authentication succeeds.
    /// * `Err(eyre::Report)` if an error occurs during authentication.
    ///
    /// # Errors
    ///
    /// Returns an error if the authentication fails.
    pub async fn authenticate(&self, password: &str) -> Result<()> {
        info!("Authenticate to the server");

        self.client.authenticate(password).await?;

        Ok(())
    }

    /// Prepares the restoration process for a specific share.
    ///
    /// # Arguments
    /// * `share` - The share to prepare for restoration.
    /// * `selection` - The list of files to restore.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the preparation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during preparation.
    ///
    /// # Errors
    ///
    /// Returns an error if the preparation fails.
    pub async fn prepare_restoration<P: AsRef<Path>>(
        &self,
        share: &str,
        selection: &[P],
    ) -> Result<u64> {
        let hostname = self.hostname.clone();
        let current_backup_id = self.current_backup_id;

        let backups = Backups::new(&self.config);
        let manifest = backups.get_manifest(&hostname, current_backup_id, share);

        let selection = selection
            .iter()
            .map(|p| p.as_ref().to_path_buf())
            .collect::<Vec<_>>();

        let entries = manifest.read_manifest_entries();
        pin_mut!(entries);

        let mut progression = BackupProgression::default();

        while let Some(entry) = entries.next().await {
            // Check if the entry is on the selection list
            let path = entry.path();
            if !selection.iter().any(|p| path.starts_with(p)) {
                continue;
            }

            progression.progress_max += entry.size();
        }

        {
            let mut progress_max = self.progress_max.lock().await;
            progress_max.insert(share.to_string(), progression.progress_max);
        }
        self.progression.lock().await.progress_max = progression.progress_max;

        self.create_event_restore_start(&[share]).await?;

        Ok(progression.progress_max)
    }

    /// Restores files for a specific share to a destination directory.
    ///
    /// # Arguments
    /// * `share` - The share to restore.
    /// * `destination_directory` - The directory to restore files to.
    /// * `selection` - The list of files to restore.
    /// * `chunk_tx` - An optional channel for sending progress updates.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the restoration succeeds.
    /// * `Err(eyre::Report)` if an error occurs during restoration.
    ///
    /// # Errors
    ///
    /// Returns an error if the restoration fails.
    pub async fn restore<P: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        share: &str,
        destination_directory: P,
        selection: &[P2],
        chunk_tx: Option<mpsc::Sender<BackupProgression>>,
    ) -> Result<()> {
        info!("Restore backup to {:?}", destination_directory.as_ref());

        let pool_path = self.pool_path.clone();
        let hostname = self.hostname.clone();
        let current_backup_id = self.current_backup_id;

        let backups = Backups::new(&self.config);
        let manifest = backups.get_manifest(&hostname, current_backup_id, share);

        let destination_directory = destination_directory.as_ref().to_path_buf();
        let selection = selection
            .iter()
            .map(|p| p.as_ref().to_path_buf())
            .collect::<Vec<_>>();

        let local_progression = BackupProgression {
            progress_max: {
                let progress_max = self.progress_max.lock().await;
                progress_max.get(share).copied().unwrap_or_default()
            },
            ..BackupProgression::default()
        };
        let local_progression = Arc::new(Mutex::new(local_progression));
        let progression = local_progression.clone();

        let (tx, rx) = tokio::sync::mpsc::channel(DEFAULT_CHANNEL_BUFFER_SIZE);

        tokio::spawn(async move {
            let entries = manifest.read_manifest_entries();
            pin_mut!(entries);

            while let Some(mut entry) = entries.next().await {
                // Check if the entry is on the selection list
                let path = entry.path();
                if !selection.iter().any(|p| path.starts_with(p)) {
                    continue;
                }

                // If continue then prefix the path with the destination directory
                let destination = destination_directory.join(path);

                entry.path = path_to_vec(&destination);

                info!("Restore file: {:?}", entry.path());

                // Send it to the client for restoration by creating a stream
                if let Err(err) = tx
                    .send(RestoreFileRequest {
                        request: Some(restore_file_request::Request::Manifest(entry.clone())),
                    })
                    .await
                {
                    error!("Failed to send restore request: {}", err);
                    break;
                }

                if entry.is_regular_file() {
                    let reader = entry.open_from_pool(&pool_path);
                    pin_mut!(reader);

                    let mut buffer = vec![0; BUFFER_SIZE];
                    while let Ok(size) = reader.read(&mut buffer).await {
                        if size == 0 {
                            break;
                        }

                        if let Err(err) = tx
                            .send(RestoreFileRequest {
                                request: Some(restore_file_request::Request::Chunk(
                                    buffer[..size].to_vec(),
                                )),
                            })
                            .await
                        {
                            error!("Failed to send restore request: {}", err);
                            break;
                        }
                    }
                }

                {
                    let mut local_progression = local_progression.lock().await;
                    local_progression.progress_current += entry.size();
                    local_progression.file_size += entry.size();
                    local_progression.file_count += 1;

                    if let Some(chunk_tx) = &chunk_tx {
                        if let Err(e) = chunk_tx.send(*local_progression).await {
                            error!("Failed to send file progression: {}", e);
                        }
                    }
                }
            }
        });

        let responses = self
            .client
            .restore_file(tokio_stream::wrappers::ReceiverStream::new(rx));
        pin_mut!(responses);

        while let Some(response) = responses.next().await {
            match response {
                Ok(reply) => {
                    if reply.restored {
                        info!("Restored file: {:?}", vec_to_path(&reply.path));
                    } else {
                        error!(
                            "Failed to restore file: {:?}, message: {}",
                            vec_to_path(&reply.path),
                            reply.message
                        );
                    }
                }
                Err(e) => {
                    error!("Error during file restoration: {}", e);
                }
            }
        }

        {
            let local_progression = progression.lock().await;

            let mut global_progression = self.progression.lock().await;
            global_progression.progress_current += local_progression.progress_current;
            global_progression.file_size += local_progression.file_size;
            global_progression.file_count += local_progression.file_count;
        }

        Ok(())
    }

    /// Closes the restoration process.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during closure.
    ///
    /// # Errors
    ///
    /// Returns an error if the closure fails.
    pub async fn close(&self) -> Result<()> {
        info!("Close restore");

        self.client.close().await?;

        self.create_event_restore_end(&[], EventStatus::Success)
            .await?;

        Ok(())
    }

    /// Creates an event indicating the start of the restoration process.
    ///
    /// # Arguments
    /// * `shares` - The shares being restored.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the event creation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during event creation.
    ///
    /// # Errors
    ///
    /// Returns an error if the event creation fails.
    pub async fn create_event_restore_start(&self, shares: &[&str]) -> Result<()> {
        let event = Event {
            id: self.uuid.clone(),
            r#type: EventType::Restore as i32,
            step: EventStep::Start as i32,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
            source: self.source as i32,
            user: String::new(),
            error_messages: Vec::new(),
            status: EventStatus::None as i32,

            information: Some(Information::Backup(EventBackupInformation {
                hostname: self.hostname.clone(),
                number: self.current_backup_id as u64,
                share_path: shares.iter().map(|s| (*s).to_string()).collect(),
            })),
        };

        append_events(&self.config.path.events_path, &[&event]).await?;

        Ok(())
    }

    /// Creates an event indicating the end of the restoration process.
    ///
    /// # Arguments
    /// * `status` - The status of the restoration process.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the event creation succeeds.
    /// * `Err(eyre::Report)` if an error occurs during event creation.
    ///
    /// # Errors
    ///
    /// Returns an error if the event creation fails.
    pub async fn create_event_restore_end(
        &self,
        shares: &[&str],
        status: EventStatus,
    ) -> Result<()> {
        let event = Event {
            id: self.uuid.clone(),
            r#type: EventType::Restore as i32,
            step: EventStep::End as i32,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
            source: self.source as i32,
            user: String::new(),
            error_messages: Vec::new(),
            status: status as i32,

            information: Some(Information::Backup(EventBackupInformation {
                hostname: self.hostname.clone(),
                number: self.current_backup_id as u64,
                share_path: shares.iter().map(|s| (*s).to_string()).collect(),
            })),
        };

        append_events(&self.config.path.events_path, &[&event]).await?;

        Ok(())
    }
}

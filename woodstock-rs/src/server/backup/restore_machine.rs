use std::{path::Path, sync::Arc};

use eyre::Result;
use tokio::sync::{mpsc, Mutex};
use tokio_util::sync::CancellationToken;
use tracing::{error, Instrument};
use uuid::Uuid;

use crate::{
    config::{
        Backups, Configuration, Context, HostConfiguration, Hosts, DEFAULT_CHANNEL_BUFFER_SIZE,
    },
    server::{client::Client, progression::BackupProgression},
};

use super::{restore::BackupRestore, restore_state::RestoreState};

pub struct ShareSelection<S: AsRef<str>, P: AsRef<Path>> {
    /// The share to restore.
    pub share: S,
    /// The list of files to restore.
    pub selection: Vec<P>,
}

pub struct RestoreBackupMachine<Clt: Client> {
    /// The client responsible for backup restoration operations.
    client: BackupRestore<Clt>,
    /// The configuration for the host involved in the restoration.
    host_configuration: HostConfiguration,
    /// The state of the restoration progression.
    progression_state: Arc<Mutex<RestoreState>>,
    /// An optional channel for sending state updates.
    state_tx: Option<mpsc::Sender<RestoreState>>,
    /// Cancelled when the user requests to stop this restore.
    cancel_token: CancellationToken,
}

impl<Clt: Client> RestoreBackupMachine<Clt> {
    /// Creates a new instance of `RestoreBackupMachine`.
    ///
    /// # Arguments
    /// * `client` - The client used for restoration.
    /// * `hostname` - The hostname of the backup to restore.
    /// * `backup_id` - The UUID v7 identifier of the backup to restore.
    /// * `backup_number` - The sequential display number of the backup.
    /// * `ctxt` - The context containing the event source.
    /// * `config` - The configuration for the backup system.
    /// * `state_tx` - An optional channel for sending state updates.
    ///
    /// # Returns
    ///
    /// * `Ok(RestoreBackupMachine)` - A new instance of `RestoreBackupMachine`.
    /// * `Err(eyre::Report)` if an error occurs during initialization.
    ///
    /// # Errors
    ///
    /// Returns an error if the initialization fails due to configuration or I/O issues.
    pub async fn new(
        client: Clt,
        hostname: &str,
        backup_id: Uuid,
        ctxt: &Context,
        state_tx: Option<mpsc::Sender<RestoreState>>,
        config: Arc<Configuration>,
        hosts: Arc<Hosts>,
        backups: Arc<Backups>,
        cancel_token: CancellationToken,
    ) -> Result<Self> {
        let client = BackupRestore::new(
            client,
            hostname,
            backup_id,
            ctxt,
            config.clone(),
            backups,
            cancel_token.clone(),
        );

        let host_configuration = hosts.get_host(hostname).await?;
        let progression_state = RestoreState::default();

        Ok(Self {
            client,
            host_configuration,

            progression_state: Arc::new(Mutex::new(progression_state)),
            state_tx,
            cancel_token,
        })
    }

    /// Sends the current progression state to the state channel.
    ///
    /// # Errors
    ///
    /// Logs an error if sending the state update fails.
    async fn send_progress(&self) {
        if let Some(state_tx) = &self.state_tx {
            let state = self.progression_state.lock().await;
            if state_tx.send(state.clone()).await.is_err() {
                error!("Failed to send state update");
            }
        }
    }

    /// Executes the authentication process.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the authentication succeeds.
    /// * `Err(eyre::Report)` if an error occurs during authentication.
    ///
    /// # Errors
    ///
    /// Returns an error if the authentication fails.
    async fn execute_authentication(&self) -> Result<()> {
        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state.start_authentication();
        }
        self.send_progress().await;

        let auth_result = self
            .client
            .authenticate(&self.host_configuration.password)
            .await;
        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state
                .process_authentication_result(auth_result)
                .inspect_err(|_| {
                    if let Some(tx) = &self.state_tx {
                        let _ = tx.try_send(progression_state.clone());
                    }
                })?;
        }

        self.send_progress().await;
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
    async fn prepare_restoration<P: AsRef<Path>>(
        &mut self,
        share: &str,
        selection: &[P],
    ) -> Result<()> {
        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state.start_prepare_restoration(share);
        }
        self.send_progress().await;

        let result = self.client.prepare_restoration(share, selection).await;
        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state
                .process_prepare_restoration_result(share, result)
                .inspect_err(|_| {
                    if let Some(tx) = &self.state_tx {
                        let _ = tx.try_send(progression_state.clone());
                    }
                })?;
        }

        self.send_progress().await;
        Ok(())
    }

    /// Restores files for a specific share to a destination directory.
    ///
    /// # Arguments
    /// * `share` - The share to restore.
    /// * `destination_directory` - The directory to restore files to.
    /// * `selection` - The list of files to restore.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the restoration succeeds.
    /// * `Err(eyre::Report)` if an error occurs during restoration.
    ///
    /// # Errors
    ///
    /// Returns an error if the restoration fails.
    async fn restore_files<P: AsRef<Path>, P2: AsRef<Path>>(
        &mut self,
        destination_directory: P,
        share: &str,
        selection: &[P2],
    ) -> Result<()> {
        {
            let mut progression_state = self.progression_state.lock().await;
            progression_state.start_restore(share);
        }
        self.send_progress().await;

        // Créer un canal pour recevoir les mises à jour de progression de la restauration
        let (progress_tx, mut progress_rx) = mpsc::channel(DEFAULT_CHANNEL_BUFFER_SIZE);

        // Spawn une tâche pour traiter les mises à jour de progression
        let progression_state_clone = self.progression_state.clone();
        let state_tx_clone = self.state_tx.clone();
        let share_clone = share.to_string();

        let restore_task = tokio::spawn(
            async move {
                while let Some(progress) = progress_rx.recv().await {
                    let mut progression_state_clone = progression_state_clone.lock().await;
                    progression_state_clone.process_restore_progress(&share_clone, &progress);
                    if let Some(state_tx) = &state_tx_clone {
                        if let Err(e) = state_tx.send(progression_state_clone.clone()).await {
                            error!("Failed to send state update during restore: {}", e);
                        }
                    }
                }
            }
            .in_current_span(),
        );

        // Appeler restore avec le canal de progression
        let result = self
            .client
            .restore(share, destination_directory, selection, Some(progress_tx))
            .await;

        // Attendre que la tâche de mise à jour de progression termine
        if let Err(e) = restore_task.await {
            error!("Error in restore progression task: {}", e);
        }

        {
            let mut progression_state = self.progression_state.lock().await;

            progression_state
                .process_restore_result(share, result)
                .inspect_err(|_| {
                    if let Some(tx) = &self.state_tx {
                        let _ = tx.try_send(progression_state.clone());
                    }
                })?;
        }
        self.send_progress().await;

        Ok(())
    }

    pub async fn progress(&self) -> BackupProgression {
        self.client.progress().await
    }

    /// Executes the restoration process.
    ///
    /// # Arguments
    /// * `share` - The share to restore.
    /// * `destination_directory` - The directory to restore files to.
    /// * `selection` - The list of files to restore.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the restoration succeeds.
    /// * `Err(eyre::Report)` if an error occurs during restoration.
    ///
    /// # Errors
    ///
    /// Returns an error if any step of the restoration process fails.
    pub async fn execute<P: AsRef<Path>, P2: AsRef<Path>>(
        &mut self,
        destination_directory: P,
        share_selection: &[ShareSelection<&str, P2>],
    ) -> Result<RestoreState> {
        self.send_progress().await;

        self.execute_authentication().await?;

        for selection in share_selection {
            let share = selection.share.as_ref();
            let selection = &selection.selection;

            self.prepare_restoration(share, selection).await?;

            // Checked here too, not just between the actual transfers below —
            // otherwise a cancel arriving during preparation (scanning every
            // manifest entry to compute progress_max for every share) would
            // wait for all shares to finish preparing before taking effect.
            if self.cancel_token.is_cancelled() {
                break;
            }
        }

        for selection in share_selection {
            // Checked before each share starts (not just after) — covers a
            // cancel that arrived during preparation above, and skips any
            // share not yet started. A share already in progress when the
            // cancel arrives stops on its own via the per-file check inside
            // `BackupRestore::restore`.
            if self.cancel_token.is_cancelled() {
                break;
            }

            let share = selection.share.as_ref();
            let selection = &selection.selection;

            self.restore_files(&destination_directory, share, selection)
                .await?;
        }

        if let Err(err) = self.client.close().await {
            error!("Error closing the connection: {}", err);
            {
                let mut progression_state = self.progression_state.lock().await;
                progression_state.set_error(format!(
                    "Erreur lors de la fermeture de la connexion: {err}",
                ));
            }
            self.send_progress().await;
            return Err(err);
        }

        {
            let mut progression_state = self.progression_state.lock().await;
            if self.cancel_token.is_cancelled() {
                progression_state.cancel();
            } else {
                progression_state.complete();
            }
        }
        self.send_progress().await;

        let state = {
            let state = self.progression_state.lock().await;
            state.clone()
        };

        Ok(state)
    }
}

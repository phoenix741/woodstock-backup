use std::collections::HashMap;

use eyre::Result;
use log::{error, info};

use crate::server::progression::BackupProgression;

#[derive(Clone, Debug)]
pub enum ErrorState {
    AuthenticationError(String),
    PreparationError(String),
    RestoreError(String),
    Unknown(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum RestoreExecutionState {
    Waiting,
    Authentication,
    Preparation(String),
    Restoring(String),
    Completed,
}

#[derive(Clone, Debug)]
pub struct ShareRestoreState {
    pub share: String,
    pub file_count: u64,
    pub restore_progression: BackupProgression,
}

#[derive(Clone, Debug)]
pub struct RestoreState {
    pub execution_state: RestoreExecutionState,
    pub error_state: Option<ErrorState>,
    pub share_states: HashMap<String, ShareRestoreState>,
    pub global_progression: BackupProgression,
}

impl Default for RestoreState {
    fn default() -> Self {
        Self {
            execution_state: RestoreExecutionState::Waiting,
            error_state: None,
            share_states: HashMap::new(),
            global_progression: BackupProgression::default(),
        }
    }
}

impl RestoreState {
    /// Starts the authentication process by updating the execution state.
    pub fn start_authentication(&mut self) {
        self.execution_state = RestoreExecutionState::Authentication;
    }

    /// Processes the result of the authentication process.
    ///
    /// # Arguments
    /// * `result` - The result of the authentication operation.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the authentication was successful.
    /// * `Err(eyre::Report)` if an error occurred during authentication.
    ///
    /// # Errors
    ///
    /// Returns an error if the authentication fails due to I/O issues.
    pub fn process_authentication_result(&mut self, result: Result<()>) -> Result<()> {
        match result {
            Ok(()) => {
                info!("Authentication successful");
                Ok(())
            }
            Err(err) => {
                error!("Error authenticating: {}", err);
                self.error_state = Some(ErrorState::AuthenticationError(err.to_string()));
                Err(err)
            }
        }
    }

    /// Starts the preparation process for a specific share.
    ///
    /// # Arguments
    /// * `share` - The share to prepare for restoration.
    pub fn start_prepare_restoration(&mut self, share: &str) {
        self.execution_state = RestoreExecutionState::Preparation(share.to_string());
    }

    /// Processes the result of the preparation process for a specific share.
    ///
    /// # Arguments
    /// * `share` - The share being prepared.
    /// * `result` - The result of the preparation operation.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the preparation was successful.
    /// * `Err(eyre::Report)` if an error occurred during preparation.
    ///
    /// # Errors
    ///
    /// Returns an error if the preparation fails due to I/O issues.
    pub fn process_prepare_restoration_result(
        &mut self,
        share: &str,
        result: Result<u64>,
    ) -> Result<()> {
        match result {
            Ok(file_count) => {
                info!("Preparation for share {} successful", share);
                self.share_states.insert(
                    share.to_string(),
                    ShareRestoreState {
                        share: share.to_string(),
                        file_count,
                        restore_progression: BackupProgression::default(),
                    },
                );
                Ok(())
            }
            Err(err) => {
                error!("Error preparing restoration for share {}: {}", share, err);
                self.error_state = Some(ErrorState::PreparationError(err.to_string()));

                Err(err)
            }
        }
    }

    /// Starts the restoration process for a specific share.
    ///
    /// # Arguments
    /// * `share` - The share to restore.
    pub fn start_restore(&mut self, share: &str) {
        self.execution_state = RestoreExecutionState::Restoring(share.to_string());
    }

    /// Updates the global progression state with the provided progression.
    ///
    /// # Arguments
    /// * `progression` - The current progression state.
    pub fn process_restore_progress(&mut self, share: &str, progression: &BackupProgression) {
        let Some(share_state) = self.share_states.get_mut(share) else {
            error!("Share {share} not found in restore state");
            return;
        };
        share_state.restore_progression = *progression;
    }

    /// Processes the result of the restoration process for a specific share.
    ///
    /// # Arguments
    /// * `share` - The share being restored.
    /// * `result` - The result of the restoration operation.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the restoration was successful.
    /// * `Err(eyre::Report)` if an error occurred during restoration.
    ///
    /// # Errors
    ///
    /// Returns an error if the restoration fails due to I/O issues.
    pub fn process_restore_result(&mut self, share: &str, result: Result<()>) -> Result<()> {
        match result {
            Ok(()) => {
                info!("Restore for share {} successful", share);
                Ok(())
            }
            Err(err) => {
                error!("Error during restore for share {}: {}", share, err);
                self.error_state = Some(ErrorState::RestoreError(err.to_string()));

                Err(err)
            }
        }
    }

    /// Marks the restoration process as completed by updating the execution state.
    pub fn complete(&mut self) {
        self.execution_state = RestoreExecutionState::Completed;
    }

    /// Sets an error state with the provided error message.
    ///
    /// # Arguments
    /// * `error_message` - The error message to set.
    pub fn set_error(&mut self, error_message: String) {
        self.error_state = Some(ErrorState::Unknown(error_message));
    }
}

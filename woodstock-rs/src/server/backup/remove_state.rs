use eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ErrorState {
    FinalizePoolRemovalError(String),
    CleanupHostRemovalStateError(String),
    BackupRemovalError(String),
    Unknown(String),
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub enum RemoveExecutionState {
    #[default]
    Waiting,
    FinalizePoolRemoval,
    CleanupHostRemovalState,
    RemovingBackup,
    Completed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoveState {
    pub execution_state: RemoveExecutionState,
    pub error_state: Option<ErrorState>,
}

impl Default for RemoveState {
    fn default() -> Self {
        Self {
            execution_state: RemoveExecutionState::Waiting,
            error_state: None,
        }
    }
}

impl RemoveState {
    /// Starts the pool removal finalization operation.
    pub fn start_finalize_pool_removal(&mut self) {
        self.execution_state = RemoveExecutionState::FinalizePoolRemoval;
    }

    /// Processes the result of the pool removal finalization operation.
    ///     
    /// # Arguments
    /// * `result` - The result of the pool removal finalization operation.
    ///
    /// # Returns
    /// * `Ok(())` if the pool removal finalization operation was successful.
    /// * `Err(eyre::Report)` if an error occurred during the operation.
    ///
    /// # Errors
    /// This function returns an error if the pool removal finalization operation fails.
    pub fn process_finalize_pool_removal_result(&mut self, result: Result<()>) -> Result<()> {
        match result {
            Ok(()) => {
                info!("Pool V3 removal finalization successful");
                Ok(())
            }
            Err(err) => {
                error!("Error during Pool V3 removal finalization: {}", err);
                self.error_state = Some(ErrorState::FinalizePoolRemovalError(err.to_string()));
                Err(err)
            }
        }
    }

    /// Starts the host removal state cleanup process by updating the execution state.
    pub fn start_cleanup_host_removal_state(&mut self) {
        self.execution_state = RemoveExecutionState::CleanupHostRemovalState;
    }

    /// Processes the result of the host removal state cleanup.
    ///
    /// # Arguments
    /// * `result` - The result of the host removal state cleanup operation.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation was successful.
    /// * `Err(eyre::Report)` if an error occurred during the operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails due to I/O issues.
    pub fn process_cleanup_host_removal_state_result(&mut self, result: Result<()>) -> Result<()> {
        match result {
            Ok(()) => {
                info!("Host removal state cleanup successful",);
                Ok(())
            }
            Err(err) => {
                error!("Error cleaning host removal state: {}", err);
                self.error_state = Some(ErrorState::CleanupHostRemovalStateError(err.to_string()));
                Err(err)
            }
        }
    }

    /// Starts the backup removal process by updating the execution state.
    pub fn start_backup_removal(&mut self) {
        self.execution_state = RemoveExecutionState::RemovingBackup;
    }

    /// Processes the result of the backup removal.
    ///
    /// # Arguments
    /// * `result` - The result of the backup removal operation.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation was successful.
    /// * `Err(eyre::Report)` if an error occurred during the operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails due to I/O issues.
    pub fn process_backup_removal_result(&mut self, result: Result<()>) -> Result<()> {
        match result {
            Ok(()) => {
                info!("Backup removal successful",);
                Ok(())
            }
            Err(err) => {
                error!("Error removing backup: {}", err);
                self.error_state = Some(ErrorState::BackupRemovalError(err.to_string()));
                Err(err)
            }
        }
    }

    /// Marks the removal process as completed by updating the execution state.
    pub fn complete(&mut self) {
        self.execution_state = RemoveExecutionState::Completed;
    }
}

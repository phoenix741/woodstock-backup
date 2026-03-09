use eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ErrorState {
    AddReferencesToPoolError(String),
    RefcntRemovalError(String),
    BackupRemovalError(String),
    Unknown(String),
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub enum RemoveExecutionState {
    #[default]
    Waiting,
    AddReferencesToPool,
    RemovingRefcnt,
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
    /// Starts the add references to pool operation.
    pub fn start_add_references_to_pool(&mut self) {
        self.execution_state = RemoveExecutionState::AddReferencesToPool;
    }

    /// Processes the result of the add references to pool operation.
    ///     
    /// # Arguments
    /// * `result` - The result of the add references to pool operation.
    ///
    /// # Returns
    /// * `Ok(())` if the add references to pool operation was successful.
    /// * `Err(eyre::Report)` if an error occurred during the operation.
    ///
    /// # Errors
    /// This function returns an error if the add references to pool operation fails.
    pub fn process_add_references_to_pool_result(&mut self, result: Result<()>) -> Result<()> {
        match result {
            Ok(()) => {
                info!("Add references to pool operation successful");
                Ok(())
            }
            Err(err) => {
                error!("Error during add references to pool operation: {}", err);
                self.error_state = Some(ErrorState::AddReferencesToPoolError(err.to_string()));
                Err(err)
            }
        }
    }

    /// Starts the reference count removal process by updating the execution state.
    pub fn start_refcnt_removal(&mut self) {
        self.execution_state = RemoveExecutionState::RemovingRefcnt;
    }

    /// Processes the result of the reference count removal.
    ///
    /// # Arguments
    /// * `result` - The result of the reference count removal operation.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation was successful.
    /// * `Err(eyre::Report)` if an error occurred during the operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails due to I/O issues.
    pub fn process_refcnt_removal_result(&mut self, result: Result<()>) -> Result<()> {
        match result {
            Ok(()) => {
                info!("Reference count removal successful",);
                Ok(())
            }
            Err(err) => {
                error!("Error removing reference counts: {}", err);
                self.error_state = Some(ErrorState::RefcntRemovalError(err.to_string()));
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

use eyre::Result;
use log::{error, info};

use crate::EventPoolCleanedInformation;

#[derive(Clone, Debug)]
pub enum ErrorState {
    ApplyingRefcntError(String),
    InitializationError(String),
    CleaningError(String),
    Unknown(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum CleanerExecutionState {
    Waiting,
    ApplyingRefcnt,
    Initialization,
    Cleaning,
    Completed,
}

#[derive(Clone, Debug, Default)]
/// Represents the progression state of the pool cleaning process.
pub struct CleanerProgression {
    /// The maximum progress value.
    pub progress_max: usize,
    /// The current progress value.
    pub progress_current: usize,
    /// The size of the files processed.
    pub file_size: u64,
    /// The size of the compressed files processed.
    pub compressed_file_size: u64,
}

#[derive(Clone, Debug)]
/// Represents the state of the pool cleaner.
pub struct CleanerState {
    /// The current execution state of the cleaner.
    pub execution_state: CleanerExecutionState,
    /// The error state, if any.
    pub error_state: Option<ErrorState>,
    /// The progression state of the cleaner.
    pub progression: CleanerProgression,
}

impl Default for CleanerState {
    fn default() -> Self {
        Self {
            execution_state: CleanerExecutionState::Waiting,
            error_state: None,
            progression: CleanerProgression::default(),
        }
    }
}

impl CleanerState {
    /// Creates a new `CleanerState` instance.
    ///
    /// # Returns
    ///
    /// A new instance of `CleanerState`.    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts the applying refcnt operations process by updating the execution state.
    pub fn start_applying_refcnt(&mut self) {
        self.execution_state = CleanerExecutionState::ApplyingRefcnt;
    }

    /// Processes the result of the applying refcnt operations process.
    ///
    /// # Arguments
    /// * `result` - The result of the refcnt operations application.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation was successful.
    /// * `Err(eyre::Report)` if an error occurred during the operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the refcnt operations application fails.
    pub fn process_applying_refcnt_result(&mut self, result: Result<()>) -> Result<()> {
        match result {
            Ok(()) => {
                info!("Applying pending refcnt operations completed successfully");
                Ok(())
            }
            Err(e) => {
                let error_message = format!("Failed to apply pending refcnt operations: {}", e);
                error!("{}", error_message);
                self.error_state = Some(ErrorState::ApplyingRefcntError(error_message));
                Err(e)
            }
        }
    }

    /// Starts the initialization process by updating the execution state.
    pub fn start_initialization(&mut self) {
        self.execution_state = CleanerExecutionState::Initialization;
    }

    /// Processes the result of the initialization process.
    ///
    /// # Arguments
    /// * `result` - The result of the initialization operation.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` if the initialization was successful.
    /// * `Err(eyre::Report)` if an error occurred during initialization.
    ///
    /// # Errors
    ///
    /// Returns an error if the initialization operation fails or if the result contains an error.
    pub fn process_initialization_result(&mut self, result: Result<usize>) -> Result<usize> {
        match result {
            Ok(max) => {
                info!(
                    "Pool cleaner initialization successful, found {} unused files",
                    max
                );
                self.progression.progress_max = max;
                Ok(max)
            }
            Err(err) => {
                error!("Error initializing pool cleaner: {}", err);
                self.error_state = Some(ErrorState::InitializationError(err.to_string()));
                Err(err)
            }
        }
    }

    /// Starts the cleaning process by updating the execution state.
    pub fn start_cleaning(&mut self) {
        self.execution_state = CleanerExecutionState::Cleaning;
    }

    /// Updates the cleaning progression state with the provided progress.
    ///
    /// # Arguments
    /// * `progress_current` - The current progress value.
    /// * `file_size` - The size of the files processed.
    /// * `compressed_file_size` - The size of the compressed files processed.
    pub fn process_cleaning_progress(
        &mut self,
        progress_current: usize,
        file_size: u64,
        compressed_file_size: u64,
    ) {
        self.progression.progress_current = progress_current;
        self.progression.file_size = file_size;
        self.progression.compressed_file_size = compressed_file_size;
    }

    /// Processes the result of the cleaning operation.
    ///
    /// # Arguments
    /// * `result` - The result of the cleaning operation.
    ///
    /// # Returns
    ///
    /// * `Ok(EventPoolCleanedInformation)` if the cleaning was successful.
    /// * `Err(eyre::Report)` if an error occurred during cleaning.
    ///
    /// # Errors
    ///
    /// Returns an error if the cleaning operation fails or if the result contains an error.
    pub fn process_cleaning_result(
        &mut self,
        result: Result<EventPoolCleanedInformation>,
    ) -> Result<EventPoolCleanedInformation> {
        match result {
            Ok(info) => {
                info!(
                    "Pool cleaning completed successfully, removed {} files ({} bytes)",
                    info.count, info.size
                );
                self.execution_state = CleanerExecutionState::Completed;
                Ok(info)
            }
            Err(err) => {
                error!("Error cleaning pool: {}", err);
                self.error_state = Some(ErrorState::CleaningError(err.to_string()));
                Err(err)
            }
        }
    }
}

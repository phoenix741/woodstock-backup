use eyre::Result;
use tracing::{error, info};

#[derive(Clone, Debug)]
/// Represents the progression of a conversion process.
pub struct ConvertProgression {
    /// The current progress value.
    pub progress_current: usize,
    /// The maximum progress value.
    pub progress_max: usize,
}

#[derive(Clone, Debug)]
/// Represents the different error states that can occur during the conversion process.
pub enum ErrorState {
    /// An error that occurs during initialization.
    InitializationError(String),
    /// An error that occurs during conversion.
    ConversionError(String),
    /// An unknown error.
    Unknown(String),
}

#[derive(Clone, Debug, PartialEq)]
/// Represents the different execution states of the conversion process.
pub enum ConvertExecutionState {
    /// The conversion process is waiting to start.
    Waiting,
    /// The conversion process is initializing.
    Initialization,
    /// The conversion process is in progress.
    Converting,
    /// The conversion process is completed.
    Completed,
}

#[derive(Clone, Debug)]
/// Represents the state of the conversion process.
pub struct ConvertState {
    /// The current execution state of the conversion process.
    pub execution_state: ConvertExecutionState,
    /// The current error state of the conversion process, if any.
    pub error_state: Option<ErrorState>,
    /// The current progression of the conversion process.
    pub global_progression: ConvertProgression,
}

impl Default for ConvertState {
    fn default() -> Self {
        Self {
            execution_state: ConvertExecutionState::Waiting,
            error_state: None,
            global_progression: ConvertProgression {
                progress_current: 0,
                progress_max: 0,
            },
        }
    }
}

impl ConvertState {
    /// Starts the initialization process by updating the execution state.
    pub fn start_initialization(&mut self) {
        self.execution_state = ConvertExecutionState::Initialization;
    }

    /// Processes the result of the initialization process.
    ///
    /// # Arguments
    /// * `result` - The result of the initialization operation.
    ///
    /// # Returns
    /// * `Ok(())` if the initialization was successful.
    /// * `Err(eyre::Report)` if an error occurred during initialization.
    ///
    /// # Errors
    /// This function returns an error if the initialization fails.
    pub fn process_initialization_result(&mut self, result: Result<usize>) -> Result<()> {
        match result {
            Ok(size) => {
                self.global_progression.progress_max = size;
                info!("Pool size calculation successful, found {} entries", size);
                Ok(())
            }
            Err(err) => {
                error!("Error calculating pool size: {}", err);
                self.error_state = Some(ErrorState::InitializationError(err.to_string()));
                Err(err)
            }
        }
    }

    /// Starts the conversion process by updating the execution state.
    pub fn start_conversion(&mut self) {
        self.execution_state = ConvertExecutionState::Converting;
    }

    /// Updates the progression state with the current progress.
    ///
    /// # Arguments
    /// * `progression` - The current progression value.
    pub fn process_conversion_progress(&mut self, progression: usize) {
        self.global_progression.progress_current = progression;
    }

    /// Processes the result of the conversion process.
    ///
    /// # Arguments
    /// * `result` - The result of the conversion operation.
    ///
    /// # Returns
    /// * `Ok(())` if the conversion was successful.
    /// * `Err(eyre::Report)` if an error occurred during the conversion.
    ///
    /// # Errors
    /// This function returns an error if the conversion fails.
    pub fn process_conversion_result(&mut self, result: Result<()>) -> Result<()> {
        match result {
            Ok(()) => {
                info!("Conversion successful");
                self.execution_state = ConvertExecutionState::Completed;
                Ok(())
            }
            Err(err) => {
                error!("Error during conversion: {}", err);
                self.error_state = Some(ErrorState::ConversionError(err.to_string()));
                Err(err)
            }
        }
    }
}

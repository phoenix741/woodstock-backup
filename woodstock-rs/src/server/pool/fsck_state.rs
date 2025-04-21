use eyre::Result;
use log::{error, info};

use crate::{pool::FsckUnusedCount, server::pool::fsck::FsckProgression};

#[derive(Clone, Debug)]
pub enum ErrorState {
    ApplyingRefcntError(String),
    InitializationError(String),
    VerifyRefcntError(String),
    VerifyUnusedError(String),
    VerifyChunkError(String),
    Unknown(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum FsckExecutionState {
    Waiting,
    ApplyingRefcnt,
    Initialization,
    VerifyRefcnt,
    VerifyUnused,
    VerifyChunk,
    Completed,
}

#[derive(Clone, Debug, Default)]
pub struct RefcntProgression {
    pub progress_max: usize,
    pub progress_current: usize,
    pub error_count: usize,
    pub total_count: usize,
}

#[derive(Clone, Debug, Default)]
pub struct UnusedProgression {
    pub progress_max: usize,
    pub progress_current: usize,
    pub in_nothing: usize,
    pub in_refcnt: usize,
    pub in_unused: usize,
    pub missing: usize,
}

#[derive(Clone, Debug, Default)]
pub struct ChunkProgression {
    pub progress_max: usize,
    pub progress_current: usize,
    pub error_count: usize,
    pub total_count: usize,
}

#[derive(Clone, Debug)]
pub struct FsckState {
    pub execution_state: FsckExecutionState,
    pub error_state: Option<ErrorState>,
    pub refcnt_progression: RefcntProgression,
    pub unused_progression: UnusedProgression,
    pub chunk_progression: ChunkProgression,
    pub dry_run: bool,
}

impl Default for FsckState {
    fn default() -> Self {
        Self {
            execution_state: FsckExecutionState::Waiting,
            error_state: None,
            refcnt_progression: RefcntProgression::default(),
            unused_progression: UnusedProgression::default(),
            chunk_progression: ChunkProgression::default(),
            dry_run: true,
        }
    }
}

impl FsckState {
    /// Creates a new `FsckState` with the specified dry-run mode.
    ///
    /// # Arguments
    /// * `dry_run` - If true, the fsck process will not modify any data.
    ///
    /// # Returns
    ///
    /// A new instance of `FsckState`.
    #[must_use]
    pub fn new(dry_run: bool) -> Self {
        Self {
            dry_run,
            ..Default::default()
        }
    }

    /// Starts the initialization process by updating the execution state.
    pub fn start_initialization(&mut self) {
        self.execution_state = FsckExecutionState::Initialization;
    }

    /// Processes the result of the initialization process.
    ///
    /// # Arguments
    /// * `refcnt_max_result` - The result of the reference count maximum calculation.
    /// * `unused_max_result` - The result of the unused maximum calculation.
    /// * `chunk_max_result` - The result of the chunk maximum calculation.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the initialization was successful.
    /// * `Err(eyre::Report)` if an error occurred during initialization.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the initialization steps (reference count, unused, or chunk maximum calculation) fails.
    pub fn process_initialization_result(
        &mut self,
        refcnt_max_result: Result<usize>,
        unused_max_result: Result<usize>,
        chunk_max_result: Result<Vec<Vec<u8>>>,
    ) -> Result<()> {
        // Process the result for refcnt_max
        match refcnt_max_result {
            Ok(max) => {
                info!(
                    "Fsck initialization for refcnt successful, found {} items to check",
                    max
                );
                self.refcnt_progression.progress_max = max;
            }
            Err(err) => {
                error!("Error initializing fsck for refcnt: {}", err);
                self.error_state = Some(ErrorState::InitializationError(format!(
                    "Failed to initialize refcnt check: {err}",
                )));
                return Err(err);
            }
        }

        // Process the result for unused_max
        match unused_max_result {
            Ok(max) => {
                info!(
                    "Fsck initialization for unused successful, found {} items to check",
                    max
                );
                self.unused_progression.progress_max = max;
            }
            Err(err) => {
                error!("Error initializing fsck for unused: {}", err);
                self.error_state = Some(ErrorState::InitializationError(format!(
                    "Failed to initialize unused check: {err}",
                )));
                return Err(err);
            }
        }

        // Process the result for chunk_max
        match chunk_max_result {
            Ok(chunks) => {
                let count = chunks.len();
                info!(
                    "Fsck initialization for chunk successful, found {} chunks to check",
                    count
                );
                self.chunk_progression.progress_max = count;
            }
            Err(err) => {
                error!("Error initializing fsck for chunk: {}", err);
                self.error_state = Some(ErrorState::InitializationError(format!(
                    "Failed to initialize chunk check: {err}",
                )));
                return Err(err);
            }
        }

        Ok(())
    }

    /// Starts the reference count verification process by updating the execution state.
    pub fn start_verify_refcnt(&mut self) {
        self.execution_state = FsckExecutionState::VerifyRefcnt;
    }

    /// Updates the reference count progression state with the provided progress.
    ///
    /// # Arguments
    /// * `progress` - The current progression state.
    pub fn process_verify_refcnt_progress(&mut self, progress: &FsckProgression) {
        self.refcnt_progression.progress_current = progress.progress_current;
        self.refcnt_progression.error_count = progress.error_count;
        self.refcnt_progression.total_count = progress.total_count;
    }

    /// Processes the result of the reference count verification process.
    ///
    /// # Arguments
    /// * `result` - The result of the reference count verification.
    ///
    /// # Returns
    ///
    /// * `Ok(EventRefCountInformation)` if the verification was successful.
    /// * `Err(eyre::Report)` if an error occurred during verification.
    ///
    /// # Errors
    ///
    /// Returns an error if the reference count verification fails or if the result contains an error.
    pub fn process_verify_refcnt_result(
        &mut self,
        result: Result<FsckProgression>,
    ) -> Result<FsckProgression> {
        match result {
            Ok(info) => {
                info!(
                    "Fsck verify refcnt completed successfully, found {}/{} errors",
                    info.error_count, info.total_count
                );

                Ok(info)
            }
            Err(err) => {
                error!("Error verifying refcnt: {}", err);
                self.error_state = Some(ErrorState::VerifyRefcntError(err.to_string()));
                Err(err)
            }
        }
    }

    /// Starts the unused verification process by updating the execution state.
    pub fn start_verify_unused(&mut self) {
        self.execution_state = FsckExecutionState::VerifyUnused;
    }

    /// Updates the unused progression state with the provided progress.
    ///
    /// # Arguments
    /// * `progress` - The current progression state.
    pub fn process_verify_unused_progress(&mut self, progress: &FsckUnusedCount) {
        self.unused_progression.progress_current =
            progress.in_nothing + progress.in_refcnt + progress.in_unused + progress.missing;
        self.unused_progression.in_nothing = progress.in_nothing;
        self.unused_progression.in_refcnt = progress.in_refcnt;
        self.unused_progression.in_unused = progress.in_unused;
        self.unused_progression.missing = progress.missing;
    }

    /// Processes the result of the unused verification process.
    ///
    /// # Arguments
    /// * `result` - The result of the unused verification.
    ///
    /// # Returns
    ///
    /// * `Ok(EventPoolInformation)` if the verification was successful.
    /// * `Err(eyre::Report)` if an error occurred during verification.
    ///
    /// # Errors
    ///
    /// Returns an error if the unused verification fails or if the result contains an error.
    pub fn process_verify_unused_result(
        &mut self,
        result: Result<FsckUnusedCount>,
    ) -> Result<FsckUnusedCount> {
        match result {
            Ok(info) => {
                info!("Fsck verify unused completed successfully, found {} in refcnt, {} in unused, {} in nothing, {} missing", 
                    info.in_refcnt, info.in_unused, info.in_nothing, info.missing);
                self.unused_progression.in_refcnt = info.in_refcnt;
                self.unused_progression.in_unused = info.in_unused;
                self.unused_progression.in_nothing = info.in_nothing;
                self.unused_progression.missing = info.missing;

                Ok(info)
            }
            Err(err) => {
                error!("Error verifying unused: {}", err);
                self.error_state = Some(ErrorState::VerifyUnusedError(err.to_string()));
                Err(err)
            }
        }
    }

    /// Starts the chunk verification process by updating the execution state.
    pub fn start_verify_chunk(&mut self) {
        self.execution_state = FsckExecutionState::VerifyChunk;
    }

    /// Updates the chunk progression state with the provided progress.
    ///
    /// # Arguments
    /// * `progress` - The current progression state.
    pub fn process_verify_chunk_progress(&mut self, progress: &FsckProgression) {
        self.chunk_progression.progress_current = progress.progress_current;
        self.chunk_progression.error_count = progress.error_count;
        self.chunk_progression.total_count = progress.total_count;
    }

    /// Processes the result of the chunk verification process.
    ///
    /// # Arguments
    /// * `result` - The result of the chunk verification.
    ///
    /// # Returns
    ///
    /// * `Ok(EventRefCountInformation)` if the verification was successful.
    /// * `Err(eyre::Report)` if an error occurred during verification.
    ///
    /// # Errors
    ///
    /// Returns an error if the chunk verification fails or if the result contains an error.
    pub fn process_verify_chunk_result(
        &mut self,
        result: Result<FsckProgression>,
    ) -> Result<FsckProgression> {
        match result {
            Ok(info) => {
                info!(
                    "Fsck verify chunk completed successfully, found {}/{} errors",
                    info.error_count, info.total_count
                );
                Ok(info)
            }
            Err(err) => {
                error!("Error verifying chunks: {}", err);
                self.error_state = Some(ErrorState::VerifyChunkError(err.to_string()));
                Err(err)
            }
        }
    }

    /// Starts the applying refcnt operations process by updating the execution state.
    pub fn start_applying_refcnt(&mut self) {
        self.execution_state = FsckExecutionState::ApplyingRefcnt;
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

    /// Completes the fsck process by updating the execution state to `Completed`.
    pub fn complete(&mut self) {
        self.execution_state = FsckExecutionState::Completed;
    }
}

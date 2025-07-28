use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
/// Represents the progression of a file list operation.
pub struct FileListProgression {
    /// The total size of the files.
    pub file_size: u64,

    /// The size of new files.
    pub new_file_size: u64,
    /// The size of modified files.
    pub modified_file_size: u64,

    /// The count of new files.
    pub new_file_count: usize,
    /// The count of modified files.
    pub modified_file_count: usize,
    /// The count of removed files.
    pub removed_file_count: usize,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
/// Represents the progression of a backup operation.
pub struct BackupProgression {
    /// The start date of the backup.
    pub start_date: DateTime<Local>,
    /// The start date of the transfer.
    pub start_transfer_date: Option<DateTime<Local>>,
    /// The end date of the transfer.
    pub end_transfer_date: Option<DateTime<Local>>,

    /// The total size of compressed files.
    pub compressed_file_size: u64,
    /// The size of new compressed files.
    pub new_compressed_file_size: u64,
    /// The size of modified compressed files.
    pub modified_compressed_file_size: u64,

    /// The total size of files.
    pub file_size: u64,
    /// The size of new files.
    pub new_file_size: u64,
    /// The size of modified files.
    pub modified_file_size: u64,

    /// The count of new files.
    pub new_file_count: usize,
    /// The total count of files.
    pub file_count: usize,
    /// The count of modified files.
    pub modified_file_count: usize,
    /// The count of removed files.
    pub removed_file_count: usize,

    /// The count of errors encountered.
    pub error_count: usize,

    /// The current progress value.
    pub progress_current: u64,
    /// The maximum progress value.
    pub progress_max: u64,
}

impl BackupProgression {
    /// Calculates the percentage of progress completed.
    ///
    /// # Returns
    ///
    /// * `f64` - The percentage of progress completed.
    #[must_use]
    pub fn percent(&self) -> f64 {
        if self.progress_max == 0 {
            return 0.0;
        }

        let per10_000 = (self.progress_current * 10_000) / self.progress_max;

        per10_000 as f64 / 100.0
    }

    /// Calculates the speed of the backup process in units per second.
    ///
    /// # Returns
    ///
    /// * `f64` - The speed of the backup process.
    #[must_use]
    pub fn speed(&self) -> f64 {
        let duration = match self.start_transfer_date {
            Some(start_transfer_date) => match self.end_transfer_date {
                Some(end_transfer_date) => (end_transfer_date - start_transfer_date).num_seconds(),
                None => (Local::now() - start_transfer_date).num_seconds(),
            },
            None => (Local::now() - self.start_date).num_seconds(),
        };

        if duration == 0 {
            return 0.0;
        }

        self.progress_current as f64 / duration as f64
    }
}

impl Default for BackupProgression {
    fn default() -> Self {
        Self {
            start_date: Local::now(),
            start_transfer_date: None,
            end_transfer_date: None,
            compressed_file_size: 0,
            new_compressed_file_size: 0,
            modified_compressed_file_size: 0,
            file_size: 0,
            new_file_size: 0,
            new_file_count: 0,
            modified_file_size: 0,
            modified_file_count: 0,
            removed_file_count: 0,
            file_count: 0,
            error_count: 0,
            progress_current: 0,
            progress_max: 0,
        }
    }
}

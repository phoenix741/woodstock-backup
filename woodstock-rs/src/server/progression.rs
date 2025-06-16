use std::time::SystemTime;

#[derive(Default, Clone, Debug)]
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

#[derive(Clone, Copy, Debug)]
/// Represents the progression of a backup operation.
pub struct BackupProgression {
    /// The start date of the backup.
    pub start_date: SystemTime,
    /// The start date of the transfer.
    pub start_transfer_date: Option<SystemTime>,
    /// The end date of the transfer.
    pub end_transfer_date: Option<SystemTime>,

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
                Some(end_transfer_date) => end_transfer_date
                    .duration_since(start_transfer_date)
                    .unwrap_or_default()
                    .as_secs_f64(),
                None => start_transfer_date
                    .elapsed()
                    .unwrap_or_default()
                    .as_secs_f64(),
            },
            None => self.start_date.elapsed().unwrap_or_default().as_secs_f64(),
        };

        if duration == 0.0 {
            return 0.0;
        }

        self.progress_current as f64 / duration
    }
}

impl Default for BackupProgression {
    fn default() -> Self {
        Self {
            start_date: SystemTime::now(),
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

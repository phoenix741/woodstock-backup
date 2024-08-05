use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct BackupProgression {
    pub start_date: SystemTime,
    pub start_transfer_date: Option<SystemTime>,
    pub end_transfer_date: Option<SystemTime>,

    pub compressed_file_size: u64,
    pub new_compressed_file_size: u64,
    pub modified_compressed_file_size: u64,

    pub file_size: u64,
    pub new_file_size: u64,
    pub modified_file_size: u64,

    pub new_file_count: usize,
    pub file_count: usize,
    pub modified_file_count: usize,
    pub removed_file_count: usize,

    pub error_count: usize,

    pub progress_current: u64,
    pub progress_max: u64,
}

impl BackupProgression {
    #[must_use]
    pub fn percent(&self) -> f64 {
        if self.progress_max == 0 {
            return 0.0;
        }

        let per10_000 = (self.progress_current * 10_000) / self.progress_max;

        per10_000 as f64 / 100.0
    }

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

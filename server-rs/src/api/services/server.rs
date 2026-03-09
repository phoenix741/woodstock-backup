//! Server management service for API business logic

use super::HostsService;
use eyre::Result;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

/// Server management service
/// Provides shared logic for server management operations
pub struct ServerService {
    hosts_service: Arc<HostsService>,
    logs_path: std::path::PathBuf,
}

impl ServerService {
    /// Create new ServerService instance
    pub fn new(hosts_service: Arc<HostsService>, logs_path: std::path::PathBuf) -> Self {
        Self {
            hosts_service,
            logs_path,
        }
    }

    /// Clear all Redis caches (host list, per-host config, backup lists).
    pub async fn clear_cache(&self) -> Result<()> {
        self.hosts_service.clear_all_caches().await;
        tracing::info!("All caches cleared");
        Ok(())
    }

    /// Get log file path
    fn get_log_file_path(&self, log_name: &str) -> std::path::PathBuf {
        self.logs_path.join(log_name)
    }

    /// Read log file content (static - entire file)
    pub async fn read_log_file(&self, log_name: &str) -> Result<String> {
        let log_path = self.get_log_file_path(log_name);

        if !log_path.exists() {
            return Ok(String::new());
        }

        let content = tokio::fs::read_to_string(&log_path).await?;
        Ok(content)
    }

    /// Get log file stream for tailing (streaming updates)
    pub async fn stream_log_file(&self, log_name: &str) -> Result<LogFileStream> {
        let log_path = self.get_log_file_path(log_name);

        if !log_path.exists() {
            return Err(eyre::eyre!(
                "Log file does not exist: {}",
                log_path.display()
            ));
        }

        let file = File::open(&log_path).await?;
        let reader = BufReader::new(file);

        Ok(LogFileStream::new(reader))
    }

    /// Get last N lines from log file
    pub async fn get_log_tail(&self, log_name: &str, lines: usize) -> Result<Vec<String>> {
        let log_path = self.get_log_file_path(log_name);

        if !log_path.exists() {
            return Ok(vec![]);
        }

        let content = tokio::fs::read_to_string(&log_path).await?;
        let lines_vec: Vec<String> = content
            .lines()
            .rev()
            .take(lines)
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        Ok(lines_vec)
    }

    /// Check if log file exists
    pub fn log_file_exists(&self, log_name: &str) -> bool {
        self.get_log_file_path(log_name).exists()
    }

    /// Get server information
    pub async fn get_server_info(&self) -> Result<ServerInfo> {
        let hosts_count = self
            .hosts_service
            .list_hosts()
            .await
            .map(|hosts| hosts.len())
            .unwrap_or(0);

        // TODO: Calculate total backups and size from all hosts
        let total_backups = 0; // Placeholder
        let total_size = 0; // Placeholder

        Ok(ServerInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            hosts_count,
            total_backups,
            total_size,
            logs_available: vec!["application.log".to_string(), "exceptions.log".to_string()],
        })
    }
}

/// Log file stream for real-time log tailing
pub struct LogFileStream {
    reader: BufReader<File>,
}

impl LogFileStream {
    fn new(reader: BufReader<File>) -> Self {
        Self { reader }
    }

    /// Read next line from log file
    pub async fn read_line(&mut self) -> Result<Option<String>> {
        let mut line = String::new();
        match self.reader.read_line(&mut line).await? {
            0 => Ok(None), // EOF
            _ => {
                if line.ends_with('\n') {
                    line.pop();
                    if line.ends_with('\r') {
                        line.pop();
                    }
                }
                Ok(Some(line))
            }
        }
    }
}

/// Server information response
#[derive(Debug, Clone, serde::Serialize)]
pub struct ServerInfo {
    pub version: String,
    pub uptime: u64,
    pub hosts_count: usize,
    pub total_backups: usize,
    pub total_size: u64,
    pub logs_available: Vec<String>,
}

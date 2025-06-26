/// # Core Configuration Module
///
/// This module defines the core configuration structures and logic for the Woodstock backup system.
/// It provides path management, configuration initialization, and global configuration handling.
///
/// ## Main Structures
///
/// - [`ConfigurationPath`]: Holds all filesystem paths used by the application.
/// - [`OptionalConfigurationPath`]: Allows overriding paths via environment variables.
/// - [`RedisConfiguration`]: Holds Redis connection settings.
/// - [`Configuration`]: Main configuration struct, including logging, cache, chunking, and paths.
/// - [`Context`]: Holds runtime context (source, username) for operations.
///
/// ## Usage
///
/// Typical usage involves creating a [`Configuration`] at startup, either with defaults or from a custom backup path.
///
/// ## Error Handling & Panics
///
/// - Most methods return `Result` and propagate I/O or parsing errors using the `eyre` crate.
/// - Some methods (e.g., `default_hostname`) may panic if system information is unavailable.
///
/// ## Thread Safety
///
/// The [`GlobalConfiguration`] static is safe for concurrent read access.
use eyre::Result;
use lazy_static::lazy_static;
use log::{info, warn, Level};
use std::{
    env,
    path::{Path, PathBuf},
};

use crate::{utils::chunk_hasher::DEFAULT_CHUNK_ALGORITHM, ChunkAlgorithm, EventSource};

#[derive(Clone, Debug)]
/// Holds all filesystem paths used by the Woodstock backup system.
pub struct ConfigurationPath {
    /// Root directory for all backup data.
    pub backup_path: PathBuf,

    /// Directory for storing SSL certificates used for secure communication.
    pub certificates_path: PathBuf,

    /// Directory for configuration files.
    pub config_path: PathBuf,

    /// Directory for storing host-specific data.
    pub hosts_path: PathBuf,

    /// Directory for storing log files.
    pub logs_path: PathBuf,

    /// Directory for the chunk pool (deduplicated file chunks).
    pub pool_path: PathBuf,

    /// Directory for the refcnt waiting to be processed.
    pub pool_refcnt_path: PathBuf,

    /// Directory for job logs and status information.
    pub jobs_path: PathBuf,

    /// Directory for event logs.
    pub events_path: PathBuf,

    /// Path to the hosts configuration file.
    pub config_path_hosts: PathBuf,

    /// Path to the scheduler configuration file.
    pub config_path_scheduler: PathBuf,

    /// Path to the statistics configuration file.
    pub config_path_statistics: PathBuf,

    /// Path to the file storing the chunk algorithm configuration.
    pub config_path_pool_algorithm: PathBuf,
}

/// Allows overriding configuration paths via environment variables.
pub struct OptionalConfigurationPath {
    pub certificates_path: Option<PathBuf>,
    pub config_path: Option<PathBuf>,
    pub hosts_path: Option<PathBuf>,
    pub logs_path: Option<PathBuf>,
    pub pool_path: Option<PathBuf>,
    pub jobs_path: Option<PathBuf>,
    pub events_path: Option<PathBuf>,
}

impl Default for OptionalConfigurationPath {
    /// Returns an [`OptionalConfigurationPath`] with values from environment variables if set.
    fn default() -> Self {
        OptionalConfigurationPath {
            certificates_path: env::var("CERTIFICATES_PATH").ok().map(PathBuf::from),
            config_path: env::var("CONFIG_PATH").ok().map(PathBuf::from),
            hosts_path: env::var("HOSTS_PATH").ok().map(PathBuf::from),
            logs_path: env::var("LOGS_PATH").ok().map(PathBuf::from),
            pool_path: env::var("POOL_PATH").ok().map(PathBuf::from),
            jobs_path: env::var("JOBS_PATH").ok().map(PathBuf::from),
            events_path: env::var("EVENTS_PATH").ok().map(PathBuf::from),
        }
    }
}

impl ConfigurationPath {
    /// Creates a new `ConfigurationPath` from a backup root and optional overrides.
    ///
    /// # Arguments
    ///
    /// * `backup_path` - The root directory for all backup data.
    /// * `optional_path` - Optional overrides for each subdirectory.
    ///
    /// # Returns
    ///
    /// A fully initialized [`ConfigurationPath`] with all subpaths set.
    #[must_use]
    pub fn new(backup_path: PathBuf, optional_path: OptionalConfigurationPath) -> Self {
        let certificates_path = optional_path
            .certificates_path
            .unwrap_or_else(|| backup_path.join("certs"));
        let config_path = optional_path
            .config_path
            .unwrap_or_else(|| backup_path.join("config"));
        let hosts_path = optional_path
            .hosts_path
            .unwrap_or_else(|| backup_path.join("hosts"));
        let logs_path = optional_path
            .logs_path
            .unwrap_or_else(|| backup_path.join("logs"));
        let pool_path = optional_path
            .pool_path
            .unwrap_or_else(|| backup_path.join("pool"));
        let jobs_path = optional_path
            .jobs_path
            .unwrap_or_else(|| logs_path.join("jobs"));
        let events_path = optional_path
            .events_path
            .unwrap_or_else(|| backup_path.join("events"));

        let config_path_hosts = config_path.join("hosts.yml");
        let config_path_scheduler = config_path.join("scheduler.yml");
        let config_path_statistics = config_path.join("statistics.yml");

        let config_path_pool_algorithm = pool_path.join("algorithm");

        let pool_refcnt_path = pool_path.join("pending");

        Self {
            backup_path,
            certificates_path,
            config_path,
            hosts_path,
            logs_path,
            pool_path,
            pool_refcnt_path,
            jobs_path,
            events_path,
            config_path_hosts,
            config_path_scheduler,
            config_path_statistics,
            config_path_pool_algorithm,
        }
    }
}

impl Default for ConfigurationPath {
    /// Returns a default [`ConfigurationPath`] using environment variables or standard locations.
    ///
    /// # Panics
    ///
    /// Panics if the current directory cannot be determined (very rare).
    fn default() -> Self {
        // Get environment variables
        let backup_path = PathBuf::from(
            env::var("BACKUP_PATH").unwrap_or_else(|_| "/var/lib/woodstock".to_string()),
        );

        ConfigurationPath::new(backup_path, OptionalConfigurationPath::default())
    }
}

#[derive(Clone, Debug)]
/// Holds Redis connection settings for the backup system.
pub struct RedisConfiguration {
    pub host: String,
    pub port: u16,
}

impl RedisConfiguration {
    /// Creates a new [`RedisConfiguration`] from host and port.
    ///
    /// # Arguments
    ///
    /// * `host` - Redis server hostname or IP.
    /// * `port` - Redis server port.
    ///
    /// # Returns
    ///
    /// A new [`RedisConfiguration`] instance.
    #[must_use]
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }
}

impl Default for RedisConfiguration {
    /// Returns a [`RedisConfiguration`] using environment variables or defaults.
    fn default() -> Self {
        RedisConfiguration {
            host: env::var("REDIS_HOST")
                .ok()
                .unwrap_or_else(|| "localhost".to_string()),
            port: env::var("REDIS_PORT")
                .ok()
                .map_or(6379, |p| p.parse().unwrap()),
        }
    }
}

#[derive(Clone, Debug)]
/// Main configuration struct for the Woodstock backup system.
///
/// Contains all runtime settings, paths, logging, cache, and chunking configuration.
pub struct Configuration {
    pub redis: RedisConfiguration,
    pub path: ConfigurationPath,
    pub log_level: Level,
    pub cache_size: usize,
    pub chunk_algorithm: ChunkAlgorithm,
}

impl Configuration {
    /// Creates a [`Configuration`] from a custom backup root path.
    ///
    /// # Arguments
    ///
    /// * `backup_path` - The root directory for all backup data.
    ///
    /// # Returns
    ///
    /// A new [`Configuration`] with all subpaths and settings initialized.
    #[must_use]
    pub fn from_backup_path(backup_path: PathBuf) -> Self {
        Self {
            path: ConfigurationPath::new(backup_path, OptionalConfigurationPath::default()),
            ..Default::default()
        }
    }

    /// Returns the Woodstock package version as a string.
    #[must_use]
    pub fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Reads the chunk algorithm from a file in the pool directory.
    ///
    /// # Arguments
    ///
    /// * `pool_path` - Path to the file containing the chunk algorithm name.
    ///
    /// # Returns
    ///
    /// * `Ok(ChunkAlgorithm)` if the file is readable and valid.
    /// * `Err(eyre::Report)` if the file is missing or invalid.
    fn read_algorithm<P: AsRef<Path>>(pool_path: P) -> Result<ChunkAlgorithm> {
        let algorithm = std::fs::read_to_string(pool_path)?;
        let algorithm = algorithm.trim();
        let algorithm = ChunkAlgorithm::from_str_name(algorithm)
            .ok_or_else(|| eyre::eyre!("Invalid chunk algorithm: {}", algorithm))?;

        Ok(algorithm)
    }

    /// Writes the current chunk algorithm to the pool directory if not already set.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the algorithm is written or already set.
    /// * `Err(eyre::Report)` if the directory or file cannot be created.
    ///
    /// # Errors
    ///
    /// Returns an error if the pool directory cannot be created or if the algorithm file cannot be written.
    pub fn fix_algorithm(&self) -> Result<()> {
        if self.path.config_path_pool_algorithm.exists() {
            return Ok(());
        }

        std::fs::create_dir_all(&self.path.pool_path)?;
        std::fs::write(
            &self.path.config_path_pool_algorithm,
            self.chunk_algorithm.as_str_name(),
        )?;

        Ok(())
    }

    /// Overwrites the chunk algorithm in the pool directory.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the algorithm is written.
    /// * `Err(eyre::Report)` if the file cannot be written.
    ///
    /// # Errors
    ///
    /// Returns an error if the algorithm file cannot be written to disk.
    pub fn overwrite_algorithm(&self) -> Result<()> {
        std::fs::write(
            &self.path.config_path_pool_algorithm,
            self.chunk_algorithm.as_str_name(),
        )?;

        Ok(())
    }
}

impl Default for Configuration {
    /// Returns a [`Configuration`] with all settings from environment variables or defaults.
    fn default() -> Self {
        let path = ConfigurationPath::default();

        let log_level = match env::var("LOG_LEVEL") {
            Ok(level) => match level.to_lowercase().as_str() {
                "error" => Level::Error,
                "warn" => Level::Warn,
                "debug" => Level::Debug,
                "trace" => Level::Trace,
                _ => Level::Info,
            },
            Err(_) => Level::Info,
        };

        let cache_size = match env::var("CACHE_SIZE") {
            Ok(size) => size.parse().unwrap_or(10),
            Err(_) => 10,
        };

        let redis = RedisConfiguration::default();

        let wanted_chunk_algorithm = match env::var("CHUNK_ALGORITHM") {
            Ok(algorithm) => match algorithm.to_lowercase().as_str() {
                "blake3" => ChunkAlgorithm::Blake3,
                "sha2_256" => ChunkAlgorithm::Sha2256,
                "sha3_256" => ChunkAlgorithm::Sha3256,
                _ => DEFAULT_CHUNK_ALGORITHM,
            },
            Err(_) => DEFAULT_CHUNK_ALGORITHM,
        };

        let chunk_algorithm = Configuration::read_algorithm(&path.config_path_pool_algorithm)
            .unwrap_or_else(|_| {
                warn!("Failed to read chunk algorithm from file, using default");
                wanted_chunk_algorithm
            });
        if chunk_algorithm != wanted_chunk_algorithm {
            warn!("Chunk algorithm in file is different from the one in environment variable");
        }

        info!(
            "Using chunk algorithm: {} (wanted: {})",
            chunk_algorithm.as_str_name(),
            wanted_chunk_algorithm.as_str_name()
        );

        Self {
            redis,
            path,
            log_level,
            cache_size,
            chunk_algorithm,
        }
    }
}

lazy_static! {
    pub static ref GlobalConfiguration: Configuration = Configuration::default();
}

///
/// The goal of the `Context` struct is to hold the configuration of the application.
/// and pass the values to the functions that need them.
#[derive(Clone, Debug)]
/// Holds runtime context for operations (source, username).
pub struct Context {
    pub source: EventSource,
    pub username: Option<String>,
}

impl Default for Context {
    /// Returns a [`Context`] with default values (CLI source, no username).
    fn default() -> Self {
        Self {
            source: EventSource::Cli,
            username: None,
        }
    }
}

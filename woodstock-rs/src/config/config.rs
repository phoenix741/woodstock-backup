use eyre::Result;
use log::{warn, Level};
use std::{
    env,
    path::{Path, PathBuf},
};

use crate::{utils::chunk_hasher::DEFAULT_CHUNK_ALGORITHM, ChunkAlgorithm, EventSource};

#[derive(Clone, Debug)]
pub struct ConfigurationPath {
    pub backup_path: PathBuf,
    pub certificates_path: PathBuf,
    pub config_path: PathBuf,

    pub hosts_path: PathBuf,
    pub logs_path: PathBuf,
    pub pool_path: PathBuf,
    pub jobs_path: PathBuf,
    pub events_path: PathBuf,

    pub config_path_hosts: PathBuf,
    pub config_path_scheduler: PathBuf,
    pub config_path_statistics: PathBuf,

    pub config_path_pool_algorithm: PathBuf,
}

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

        Self {
            backup_path,
            certificates_path,
            config_path,
            hosts_path,
            logs_path,
            events_path,
            pool_path,
            jobs_path,

            config_path_hosts,
            config_path_scheduler,
            config_path_statistics,

            config_path_pool_algorithm,
        }
    }
}

impl Default for ConfigurationPath {
    fn default() -> Self {
        // Get environment variables
        let backup_path = PathBuf::from(
            env::var("BACKUP_PATH").unwrap_or_else(|_| "/var/lib/woodstock".to_string()),
        );

        ConfigurationPath::new(backup_path, OptionalConfigurationPath::default())
    }
}

#[derive(Clone, Debug)]

pub struct RedisConfiguration {
    pub host: String,
    pub port: u16,
}

impl RedisConfiguration {
    #[must_use]
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }
}

impl Default for RedisConfiguration {
    fn default() -> Self {
        RedisConfiguration {
            host: env::var("REDIS_HOST")
                .ok()
                .unwrap_or_else(|| "localhost".to_string()),
            port: env::var("REDIS_PORT")
                .ok()
                .map(|p| p.parse().unwrap())
                .unwrap_or(6379),
        }
    }
}

#[derive(Clone, Debug)]

pub struct Configuration {
    pub redis: RedisConfiguration,
    pub path: ConfigurationPath,
    pub log_level: Level,
    pub cache_size: usize,
    pub chunk_algorithm: ChunkAlgorithm,
}

impl Configuration {
    #[must_use]
    pub fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn read_algorithm<P: AsRef<Path>>(pool_path: P) -> Result<ChunkAlgorithm> {
        let algorithm = std::fs::read_to_string(pool_path)?;
        let algorithm = ChunkAlgorithm::from_str_name(&algorithm)
            .ok_or_else(|| eyre::eyre!("Invalid chunk algorithm: {}", algorithm))?;

        Ok(algorithm)
    }

    /// Write the algorithm to the pool directory to ensure that we can't change it without conversion
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
}

impl Default for Configuration {
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

        Self {
            redis,
            path,
            log_level,
            cache_size,
            chunk_algorithm,
        }
    }
}

///
/// The goal of the `Context` struct is to hold the configuration of the application.
/// and pass the values to the functions that need them.
#[derive(Clone, Debug)]
pub struct Context {
    pub config: Configuration,
    pub source: EventSource,
    pub username: Option<String>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            config: Configuration::default(),
            source: EventSource::Cli,
            username: None,
        }
    }
}

impl Context {
    #[must_use]
    pub fn from_backup_path(
        backup_path: PathBuf,
        source: EventSource,
        username: Option<&str>,
    ) -> Self {
        Self {
            config: Configuration {
                path: ConfigurationPath::new(backup_path, OptionalConfigurationPath::default()),
                ..Default::default()
            },
            source,
            username: username.map(|s| s.to_string()),
        }
    }
}

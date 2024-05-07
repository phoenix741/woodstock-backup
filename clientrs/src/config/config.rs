use log::Level;
use std::{env, path::PathBuf};

#[derive(Clone)]

pub struct ConfigurationPath {
    pub backup_path: PathBuf,
    pub certificates_path: PathBuf,
    pub config_path: PathBuf,
    pub hosts_path: PathBuf,
    pub logs_path: PathBuf,
    pub pool_path: PathBuf,
    pub jobs_path: PathBuf,

    pub config_path_hosts: PathBuf,
    pub config_path_scheduler: PathBuf,
    pub config_path_tools: PathBuf,
}

impl ConfigurationPath {
    #[must_use]
    pub fn new(
        backup_path: PathBuf,
        certificates_path: Option<PathBuf>,
        config_path: Option<PathBuf>,
        hosts_path: Option<PathBuf>,
        logs_path: Option<PathBuf>,
        pool_path: Option<PathBuf>,
        jobs_path: Option<PathBuf>,
    ) -> Self {
        let certificates_path = certificates_path.unwrap_or_else(|| backup_path.join("certs"));
        let config_path = config_path.unwrap_or_else(|| backup_path.join("config"));
        let hosts_path = hosts_path.unwrap_or_else(|| backup_path.join("hosts"));
        let logs_path = logs_path.unwrap_or_else(|| backup_path.join("logs"));
        let pools_path = pool_path.unwrap_or_else(|| backup_path.join("pool"));
        let jobs_path = jobs_path.unwrap_or_else(|| logs_path.join("jobs"));

        let config_path_hosts = config_path.join("hosts.yml");
        let config_path_scheduler = config_path.join("scheduler.yml");
        let config_path_tools = config_path.join("tools.yml");

        Self {
            backup_path,
            certificates_path,
            config_path,
            hosts_path,
            logs_path,
            pool_path: pools_path,
            jobs_path,

            config_path_hosts,
            config_path_scheduler,
            config_path_tools,
        }
    }
}

impl Default for ConfigurationPath {
    fn default() -> Self {
        // Get environment variables
        let backup_path = PathBuf::from(
            env::var("BACKUP_PATH").unwrap_or_else(|_| "/var/lib/woodstock".to_string()),
        );

        ConfigurationPath::new(
            backup_path,
            env::var("CERTIFICATES_PATH").ok().map(PathBuf::from),
            env::var("CONFIG_PATH").ok().map(PathBuf::from),
            env::var("HOSTS_PATH").ok().map(PathBuf::from),
            env::var("LOGS_PATH").ok().map(PathBuf::from),
            env::var("POOL_PATH").ok().map(PathBuf::from),
            env::var("JOBS_PATH").ok().map(PathBuf::from),
        )
    }
}

#[derive(Clone)]

pub struct Configuration {
    pub path: ConfigurationPath,
    pub log_level: Level,
}

impl Configuration {
    #[must_use]
    pub fn new(backup_path: PathBuf, log_level: Level) -> Self {
        Self {
            path: ConfigurationPath::new(backup_path, None, None, None, None, None, None),
            log_level,
        }
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

        Self { path, log_level }
    }
}

///
/// The goal of the `Context` struct is to hold the configuration of the application.
/// and pass the values to the functions that need them.
#[derive(Default, Clone)]
pub struct Context {
    pub config: Configuration,
}

impl Context {
    #[must_use]
    pub fn new(backup_path: PathBuf, log_level: Level) -> Self {
        Self {
            config: Configuration::new(backup_path, log_level),
        }
    }
}

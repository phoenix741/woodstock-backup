use log::Level;
use std::{env, path::PathBuf};

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

impl Default for ConfigurationPath {
    fn default() -> Self {
        // Get environment variables
        let backup_path = PathBuf::from(
            env::var("BACKUP_PATH").unwrap_or_else(|_| "/var/lib/woodstock".to_string()),
        );
        let certificates_path = match env::var("CERTIFICATES_PATH") {
            Ok(path) => PathBuf::from(path),
            Err(_) => backup_path.join("certs"),
        };
        let config_path = match env::var("CONFIG_PATH") {
            Ok(path) => PathBuf::from(path),
            Err(_) => backup_path.join("config"),
        };
        let hosts_path = match env::var("HOSTS_PATH") {
            Ok(path) => PathBuf::from(path),
            Err(_) => backup_path.join("hosts"),
        };
        let logs_path = match env::var("LOGS_PATH") {
            Ok(path) => PathBuf::from(path),
            Err(_) => backup_path.join("logs"),
        };
        let pools_path = match env::var("POOL_PATH") {
            Ok(path) => PathBuf::from(path),
            Err(_) => backup_path.join("pool"),
        };
        let jobs_path = match env::var("JOBS_PATH") {
            Ok(path) => PathBuf::from(path),
            Err(_) => logs_path.join("jobs"),
        };

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

pub struct Configuration {
    pub path: ConfigurationPath,
    pub log_level: Level,
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

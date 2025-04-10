use log::Level;
use woodstock::{
  config::{Configuration, ConfigurationPath, RedisConfiguration},
  ChunkAlgorithm,
};

#[napi(object)]
pub struct JsConfigurationPath {
  pub backup_path: String,
  pub certificates_path: String,
  pub config_path: String,

  pub hosts_path: String,
  pub logs_path: String,
  pub pool_path: String,
  pub jobs_path: String,
  pub events_path: String,

  pub config_path_hosts: String,
  pub config_path_scheduler: String,
  pub config_path_statistics: String,
}

impl From<ConfigurationPath> for JsConfigurationPath {
  fn from(path: ConfigurationPath) -> Self {
    JsConfigurationPath {
      backup_path: path.backup_path.to_string_lossy().to_string(),
      certificates_path: path.certificates_path.to_string_lossy().to_string(),
      config_path: path.config_path.to_string_lossy().to_string(),
      hosts_path: path.hosts_path.to_string_lossy().to_string(),
      logs_path: path.logs_path.to_string_lossy().to_string(),
      pool_path: path.pool_path.to_string_lossy().to_string(),
      jobs_path: path.jobs_path.to_string_lossy().to_string(),
      events_path: path.events_path.to_string_lossy().to_string(),

      config_path_hosts: path.config_path_hosts.to_string_lossy().to_string(),
      config_path_scheduler: path.config_path_scheduler.to_string_lossy().to_string(),
      config_path_statistics: path.config_path_statistics.to_string_lossy().to_string(),
    }
  }
}

#[napi(object)]
pub struct JsRedisConfiguration {
  pub host: String,
  pub port: u16,
}

impl From<RedisConfiguration> for JsRedisConfiguration {
  fn from(redis: RedisConfiguration) -> Self {
    JsRedisConfiguration {
      host: redis.host.to_string(),
      port: redis.port,
    }
  }
}

#[napi]
pub enum JsLogLevel {
  Error = 1,
  Warn,
  Info,
  Debug,
  Trace,
}

impl From<Level> for JsLogLevel {
  fn from(level: Level) -> Self {
    match level {
      Level::Error => JsLogLevel::Error,
      Level::Warn => JsLogLevel::Warn,
      Level::Info => JsLogLevel::Info,
      Level::Debug => JsLogLevel::Debug,
      Level::Trace => JsLogLevel::Trace,
    }
  }
}

#[napi]
pub enum JsChunkAlgorithm {
  Sha3_256,
  Sha2_256,
  Blake3,
}

impl From<ChunkAlgorithm> for JsChunkAlgorithm {
  fn from(algorithm: ChunkAlgorithm) -> Self {
    match algorithm {
      ChunkAlgorithm::Sha3256 => JsChunkAlgorithm::Sha3_256,
      ChunkAlgorithm::Sha2256 => JsChunkAlgorithm::Sha2_256,
      ChunkAlgorithm::Blake3 => JsChunkAlgorithm::Blake3,
    }
  }
}

#[napi(object)]
pub struct JsConfiguration {
  pub redis: JsRedisConfiguration,
  pub path: JsConfigurationPath,
  pub log_level: JsLogLevel,
  pub cache_size: u32,
  pub chunk_algorithm: JsChunkAlgorithm,
  pub version: String,
}

impl From<Configuration> for JsConfiguration {
  fn from(config: Configuration) -> Self {
    JsConfiguration {
      redis: config.redis.into(),
      path: config.path.into(),
      log_level: config.log_level.into(),
      cache_size: config.cache_size as u32,
      chunk_algorithm: config.chunk_algorithm.into(),
      version: Configuration::version(),
    }
  }
}

#[napi]
#[must_use]
pub fn get_configuration() -> JsConfiguration {
  Configuration::default().into()
}

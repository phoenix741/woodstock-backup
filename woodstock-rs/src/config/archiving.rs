use std::sync::Arc;

use eyre::Result;
use serde::{Deserialize, Serialize};
use tokio::fs::read_to_string;
use tracing::debug;

use crate::utils::path::list_to_globset;

use super::Configuration;

/// A single named archiving policy: which hosts, on what schedule, to which
/// destination, in which format.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveProfile {
    /// Unique name, also used as the on-disk/job identifier.
    pub name: String,
    pub host_selection: HostSelection,
    pub schedule_cron: String,
    pub destination: std::path::PathBuf,
    pub format: ArchiveFormat,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum HostSelection {
    All,
    Glob { pattern: String },
    Include { hosts: Vec<String> },
    Exclude { hosts: Vec<String> },
}

impl HostSelection {
    /// Resolves this selection against the full list of known hostnames.
    ///
    /// # Errors
    ///
    /// Returns an error if the `Glob` variant's pattern cannot be parsed.
    pub fn resolve(&self, all_hosts: &[String]) -> Result<Vec<String>> {
        let selected = match self {
            HostSelection::All => all_hosts.to_vec(),
            HostSelection::Glob { pattern } => {
                let globset = list_to_globset(&[pattern.as_str()])?;
                all_hosts
                    .iter()
                    .filter(|host| globset.is_match(host.as_str()))
                    .cloned()
                    .collect()
            }
            HostSelection::Include { hosts } => all_hosts
                .iter()
                .filter(|host| hosts.contains(host))
                .cloned()
                .collect(),
            HostSelection::Exclude { hosts } => all_hosts
                .iter()
                .filter(|host| !hosts.contains(host))
                .cloned()
                .collect(),
        };
        Ok(selected)
    }
}

/// Options only meaningful for tar-family formats — carried as the payload of
/// their [`ArchiveFormat`] variant so `Dir` cannot represent them at all.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TarOptions {
    /// Also write a `sha256sum`-compatible checksum file.
    #[serde(default)]
    pub checksum: bool,

    /// Compression level passed to the codec selected by the enclosing
    /// `ArchiveFormat` variant; when omitted, each codec uses its own
    /// recommended default (gzip/xz: 6, zstd: 3). Out-of-range values are
    /// clamped to the codec's valid range rather than rejected, so a typo in
    /// `archiving.yml` degrades gracefully instead of failing a scheduled run.
    #[serde(default)]
    pub compression_level: Option<i32>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ArchiveFormat {
    Tar(TarOptions),
    TarGz(TarOptions),
    TarXz(TarOptions),
    TarZstd(TarOptions),
    Dir,
}

impl ArchiveFormat {
    /// The tar-family options carried by this format, or `None` for `Dir`.
    #[must_use]
    pub fn tar_options(self) -> Option<TarOptions> {
        match self {
            Self::Tar(o) | Self::TarGz(o) | Self::TarXz(o) | Self::TarZstd(o) => Some(o),
            Self::Dir => None,
        }
    }
}

/// Service to load archiving profiles from `archiving.yml`, re-read fresh on
/// every call (same hot-reload-by-re-read pattern as [`super::Scheduler`]).
pub struct ArchivingConfig {
    config: Arc<Configuration>,
}

impl ArchivingConfig {
    #[must_use]
    pub fn new(config: Arc<Configuration>) -> Self {
        Self { config }
    }

    pub async fn list_profiles(&self) -> Result<Vec<ArchiveProfile>> {
        let content = read_to_string(&self.config.path.config_path_archiving).await;
        let profiles = match content {
            Ok(data) => serde_yaml_ng::from_str::<Vec<ArchiveProfile>>(&data)?,
            Err(_) => {
                debug!("Archiving profiles file missing, no profile configured");
                Vec::new()
            }
        };
        Ok(profiles)
    }

    pub async fn get_profile(&self, name: &str) -> Result<Option<ArchiveProfile>> {
        let profiles = self.list_profiles().await?;
        Ok(profiles.into_iter().find(|profile| profile.name == name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hosts() -> Vec<String> {
        vec!["srv-web".into(), "srv-db".into(), "laptop-alice".into()]
    }

    #[test]
    fn resolve_all() {
        let selection = HostSelection::All;
        assert_eq!(selection.resolve(&hosts()).unwrap(), hosts());
    }

    #[test]
    fn resolve_glob() {
        let selection = HostSelection::Glob {
            pattern: "srv-*".into(),
        };
        let mut resolved = selection.resolve(&hosts()).unwrap();
        resolved.sort();
        assert_eq!(resolved, vec!["srv-db".to_string(), "srv-web".to_string()]);
    }

    #[test]
    fn resolve_glob_matching_nothing() {
        let selection = HostSelection::Glob {
            pattern: "nope-*".into(),
        };
        assert!(selection.resolve(&hosts()).unwrap().is_empty());
    }

    #[test]
    fn resolve_include() {
        let selection = HostSelection::Include {
            hosts: vec!["srv-db".into()],
        };
        assert_eq!(
            selection.resolve(&hosts()).unwrap(),
            vec!["srv-db".to_string()]
        );
    }

    #[test]
    fn resolve_include_empty_list() {
        let selection = HostSelection::Include { hosts: vec![] };
        assert!(selection.resolve(&hosts()).unwrap().is_empty());
    }

    #[test]
    fn resolve_exclude() {
        let selection = HostSelection::Exclude {
            hosts: vec!["srv-db".into()],
        };
        let mut resolved = selection.resolve(&hosts()).unwrap();
        resolved.sort();
        assert_eq!(
            resolved,
            vec!["laptop-alice".to_string(), "srv-web".to_string()]
        );
    }

    #[test]
    fn yaml_round_trip() {
        let yaml = r"
- name: usb-weekly
  hostSelection: { mode: all }
  scheduleCron: '0 0 3 * * SAT *'
  destination: /media/disqueUsb1
  format: { type: tar_gz, checksum: true, compressionLevel: 9 }
  enabled: true
- name: nas-monthly
  hostSelection: { mode: glob, pattern: 'srv-*' }
  scheduleCron: '0 0 4 1 * * *'
  destination: /mnt/nas/woodstock-archive
  format: { type: dir }
- name: usb-zstd
  hostSelection: { mode: all }
  scheduleCron: '0 0 3 * * SUN *'
  destination: /media/disqueUsb2
  format: { type: tar_zstd }
";
        let profiles: Vec<ArchiveProfile> = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(profiles.len(), 3);
        assert_eq!(profiles[0].name, "usb-weekly");
        assert_eq!(
            profiles[0].format,
            ArchiveFormat::TarGz(TarOptions {
                checksum: true,
                compression_level: Some(9)
            })
        );
        assert_eq!(
            profiles[0].format.tar_options(),
            Some(TarOptions {
                checksum: true,
                compression_level: Some(9)
            })
        );
        // enabled defaults when omitted, checksum/compressionLevel default within TarOptions
        assert!(profiles[1].enabled);
        assert_eq!(profiles[1].format, ArchiveFormat::Dir);
        assert_eq!(profiles[1].format.tar_options(), None);
        assert_eq!(
            profiles[2].format,
            ArchiveFormat::TarZstd(TarOptions::default())
        );
    }

    #[tokio::test]
    async fn list_profiles_missing_file_returns_empty() {
        let mut config = Configuration::default();
        config.path.config_path_archiving = std::path::PathBuf::from("/nonexistent/archiving.yml");
        let archiving = ArchivingConfig::new(Arc::new(config));
        assert!(archiving.list_profiles().await.unwrap().is_empty());
    }
}

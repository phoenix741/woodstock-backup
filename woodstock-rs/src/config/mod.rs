//! # Configuration Module
//!
//! The `config` module provides a comprehensive configuration system for the Woodstock backup
//! application. It handles all aspects of configuration, from file paths and runtime settings
//! to backup scheduling and host management.
//!
//! ## Module Structure
//!
//! This module is organized into several sub-modules, each responsible for a specific aspect
//! of configuration:
//!
//! * `backups` - Manages backup configuration and metadata
//! * `core` - Provides core configuration structures and logic
//! * `constants` - Defines system-wide constants
//! * `hosts` - Handles host configuration
//! * `model` - Defines data models for configuration entities
//!
//! ## Key Features
//!
//! * Environment variable support for overriding configuration values
//! * Default configuration generation when configuration files are missing
//! * Configuration validation
//! * Path management for all application directories
//! * Backup scheduling configuration
//! * Host configuration with operations and tasks
//!
//! ## Usage
//!
//! The configuration system is typically initialized at application startup:
//!
//! Most components of the application will receive a reference to the configuration
//! or specific parts of it as needed.

/// Module for archiving profiles configuration management.
mod archiving;

/// Module for backup configuration management.
mod backups;

/// Module for blackout window configuration and evaluation.
mod blackout;

/// Module providing core configuration structures and functionality.
mod core;

/// Module defining system-wide constants.
mod constants;

/// Module for host configuration management.
mod hosts;

/// Module for scheduling configuration management
mod scheduler;

/// Module defining data models for configuration entities.
mod model;

pub use archiving::{ArchiveFormat, ArchiveProfile, ArchivingConfig, HostSelection, TarOptions};
pub use backups::{
    BackupChangedEvent, Backups, ShareRecord, ShareSnapshotMethod, BACKUP_CHANGED_CHANNEL,
};
pub use blackout::{blackout_status_at, BlackoutWindow};
pub use constants::{
    BUFFER_SIZE, CHUNK_SIZE, CHUNK_SIZE_U64, DEFAULT_BACKUP_TIMEOUT_SECS,
    DEFAULT_CHANNEL_BUFFER_SIZE, DEFAULT_PORT, DNS_RESOLVE_MAX_CONCURRENCY,
    DNS_RESOLVE_TIMEOUT_SEC, FSCK_PROGRESS_BATCH_SIZE, HOST_ONLINE_CHANNEL, MDNS_SERVICE_NAME,
    MDNS_SUFFIX, MDNS_TIMEOUT_MSEC, REDIS_WOODSTOCK_KEY_DNS,
};
pub use core::{
    Configuration, ConfigurationPath, Context, OptionalConfigurationPath, RedisConfiguration,
    GLOBAL_CONFIGURATION,
};
pub use hosts::Hosts;
pub use model::{
    ApplicationScheduler, Backup, BackupOperation, BackupStatus, BackupTaskShare,
    ExecuteCommandOperation, FailedStatus, FinishingStatus, HostConfigOperation, HostConfiguration,
    RemovingStatus, Schedule, ScheduledBackupToKeep,
};
pub use scheduler::Scheduler;

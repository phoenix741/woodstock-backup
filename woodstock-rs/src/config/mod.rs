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

/// Module for backup configuration management.
mod backups;

/// Module providing core configuration structures and functionality.
mod core;

/// Module defining system-wide constants.
mod constants;

/// Module for host configuration management.
mod hosts;

/// Module defining data models for configuration entities.
mod model;

pub use backups::*;
pub use constants::*;
pub use core::*;
pub use hosts::*;
pub use model::*;

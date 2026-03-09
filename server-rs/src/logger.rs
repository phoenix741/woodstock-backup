//! Logging module for server-rs
//!
//! Provides structured logging with request tracing, metrics collection,
//! and integration with the existing Woodstock logging infrastructure.

use std::path::Path;

use eyre::Result;
use tracing::debug;
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::jobs::layers::job_log::JobLogLayer;

/// Initialize logging for the server
pub fn init_logging<P: AsRef<Path>>(log_path: P, log_filename: &str) -> Result<()> {
    // Get log level from environment or default to INFO
    let log_level = std::env::var("LOG_LEVEL")
        .unwrap_or_else(|_| "info,server_rs=debug,woodstock_rs=debug".to_string());

    // Create filter from environment
    let env_filter =
        EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new(&log_level))?;

    // Console output layer with pretty formatting
    let console_layer = tracing_subscriber::fmt::layer();

    let file_appender = tracing_appender::rolling::daily(log_path, log_filename);
    let (file_appender, _guard) = tracing_appender::non_blocking(file_appender);

    // File output layer for production logs
    let file_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(false)
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_ansi(false)
        .json()
        .with_writer(file_appender);

    // Build subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    debug!("Logging initialized for server-rs");
    Ok(())
}

/// Initialize logging for the job_worker, with an additional [`JobLogLayer`]
/// that writes per-job log files alongside the global rolling file.
pub fn init_job_worker_logging<P: AsRef<Path>>(
    log_path: P,
    log_filename: &str,
    job_log_layer: JobLogLayer,
) -> Result<()> {
    let log_level = std::env::var("LOG_LEVEL")
        .unwrap_or_else(|_| "info,server_rs=debug,woodstock_rs=debug".to_string());

    let env_filter =
        EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new(&log_level))?;

    let console_layer = tracing_subscriber::fmt::layer();

    let file_appender = tracing_appender::rolling::daily(log_path, log_filename);
    let (file_appender, _guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(false)
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_ansi(false)
        .json()
        .with_writer(file_appender);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .with(job_log_layer)
        .init();

    debug!("Logging initialized for job_worker with per-job file logging");
    Ok(())
}

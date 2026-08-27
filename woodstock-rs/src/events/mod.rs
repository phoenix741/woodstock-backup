//! # Events Module
//!
//! This module provides event logging and management for the Woodstock backup system.
//! It allows appending, reading, and formatting events related to backup operations, including
//! backup start, end, and removal. Events are stored in protobuf files, organized by date, and
//! protected by file locks to ensure consistency in concurrent environments.
//!
//! ## Main Functions
//!
//! - [`append_events`]: Append one or more events to the event log for the current day.
//! - [`read_events`]: Read all events between two dates (inclusive).
//! - [`create_event_backup_start`]: Log the start of a backup operation.
//! - [`create_event_backup_end`]: Log the end of a backup operation.
//! - [`create_event_backup_remove`]: Log the removal of a backup.
//!
//! ## Error Handling & Panics
//!
//! - All public functions return `Result` and propagate I/O or serialization errors using the `eyre` crate.
//! - Panics are not expected under normal operation; errors are returned as `Result`.
//!
//! ## Thread Safety
//!
//! File locks are used to ensure safe concurrent access to event files.
//!
//! ## See Also
//!
//! - [`Event`], [`EventType`], [`EventStatus`], [`EventSource`]: Event data structures
//! - [`ProtobufWriter`], [`ProtobufReader`]: For serialization

use std::fmt::{self};
use std::{path::Path, time::SystemTime};

use chrono::NaiveDate;
use eyre::Result;
use tokio::fs;
use uuid::Uuid;

use crate::{
    config::Configuration,
    proto::{ProtobufReader, ProtobufWriter, UnCompressedWriter},
    utils::lock_redis::{LockOperation, PoolLockRedis},
    woodstock::event::Information,
    Event, EventBackupInformation, EventSource, EventStatus, EventStep, EventType,
};

/// Appends one or more events to the event log for the current day.
///
/// # Arguments
///
/// * `config` - The application configuration.
/// * `path` - The directory where event files are stored.
/// * `events` - Slice of references to events to append.
///
/// # Returns
///
/// * `Ok(())` if the events were appended successfully.
/// * `Err(eyre::Report)` if the directory cannot be created or the file cannot be written.
///
/// # Errors
///
/// Returns an error if the directory cannot be created, the file cannot be locked, or writing fails.
///
/// # Panics
///
/// This function does not panic under normal operation.
pub async fn append_events<P: AsRef<Path>>(
    config: &Configuration,
    path: P,
    events: &[&Event],
) -> Result<()> {
    let path = path.as_ref();
    let lockfilename = path.with_file_name("lock");

    // Create the directory if it does not exist
    fs::create_dir_all(path).await?;

    let _lock =
        PoolLockRedis::new_with_path(&config.redis_url(), &lockfilename, LockOperation::Events)
            .await?
            .lock_exclusive()
            .await?;

    // Get the current date
    let current_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let daily_path = path.join(format!("{current_date}.events"));

    let mut writer = ProtobufWriter::<UnCompressedWriter, Event>::open(daily_path).await?;

    for event in events {
        writer.write(event).await?;
    }

    writer.flush().await?;

    Ok(())
}

/// Generates a list of date strings in the format "YYYY-MM-DD" between two dates (inclusive).
///
/// # Arguments
///
/// * `start_date` - The start date (inclusive).
/// * `end_date` - The end date (inclusive).
///
/// # Returns
///
/// A vector of date strings for each day in the range.
fn list_date(start_date: NaiveDate, end_date: NaiveDate) -> Vec<String> {
    let mut dates = Vec::new();
    let mut current_date = start_date;
    while current_date <= end_date {
        let current_date_str = current_date.format("%Y-%m-%d").to_string();

        dates.push(current_date_str);
        current_date += chrono::Duration::days(1);
    }

    dates
}

/// Reads all events between two dates (inclusive).
///
/// # Arguments
///
/// * `config` - The application configuration.
/// * `path` - The directory where event files are stored.
/// * `start_date` - The start date (inclusive).
/// * `end_data` - The end date (inclusive).
///
/// # Returns
///
/// * `Ok(Vec<Event>)` - All events found in the date range.
/// * `Err(eyre::Report)` if reading or parsing fails.
///
/// # Errors
///
/// Returns an error if a file cannot be read or parsed.
///
/// # Panics
///
/// This function does not panic under normal operation.
pub async fn read_events<P: AsRef<Path>>(
    config: &Configuration,
    path: P,
    start_date: NaiveDate,
    end_data: NaiveDate,
) -> Result<Vec<Event>> {
    let lockfilename = path.as_ref().with_extension("lock");
    let _lock =
        PoolLockRedis::new_with_path(&config.redis_url(), &lockfilename, LockOperation::Events)
            .await?
            .lock_exclusive()
            .await?;

    let dates = list_date(start_date, end_data);

    let mut events = Vec::new();

    for date in dates {
        let daily_path = path.as_ref().join(format!("{date}.events"));
        if !daily_path.exists() {
            continue;
        }

        let mut reader = ProtobufReader::<Event>::new(daily_path, false).await?;
        reader.read_to_end(&mut events).await?;
    }

    Ok(events)
}

/// Logs the start of a backup operation as an event.
///
/// # Arguments
///
/// * `config` - The application configuration.
/// * `path` - The directory where event files are stored.
/// * `uuid` - Unique identifier for the backup operation.
/// * `source` - Source of the event (CLI, daemon, etc.).
/// * `hostname` - Hostname for which the backup is started.
/// * `num` - Backup number.
/// * `shares` - List of share paths involved in the backup.
///
/// # Returns
///
/// * `Ok(())` if the event was logged successfully.
/// * `Err(eyre::Report)` if writing fails.
///
/// # Errors
///
/// Returns an error if writing fails.
///
/// # Panics
///
/// This function does not panic under normal operation.
pub async fn create_event_backup_start<P: AsRef<Path>>(
    config: &Configuration,
    path: P,
    uuid: &[u8],
    source: EventSource,
    hostname: &str,
    backup_id: Uuid,
    num: usize,
    shares: &[&str],
) -> Result<()> {
    let event = Event {
        id: uuid.to_vec(),
        event_type: EventType::Backup as i32,
        step: EventStep::Start as i32,
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs(),
        source: source as i32,
        user: String::new(),
        error_messages: Vec::new(),
        status: EventStatus::None as i32,

        information: Some(Information::Backup(EventBackupInformation {
            hostname: hostname.to_string(),
            number: num as u64,
            share_path: shares.iter().map(|s| (*s).to_string()).collect(),
            id: backup_id.to_string(),
        })),
    };

    append_events(config, path, &[&event]).await?;

    Ok(())
}

/// Logs the end of a backup operation as an event.
///
/// # Arguments
///
/// * `config` - The application configuration.
/// * `path` - The directory where event files are stored.
/// * `id` - Unique identifier for the backup operation.
/// * `source` - Source of the event (CLI, daemon, etc.).
/// * `hostname` - Hostname for which the backup ended.
/// * `num` - Backup number.
/// * `shares` - List of share paths involved in the backup.
/// * `status` - Final status of the backup operation.
///
/// # Returns
///
/// * `Ok(())` if the event was logged successfully.
/// * `Err(eyre::Report)` if writing fails.
///
/// # Errors
///
/// Returns an error if writing fails.
///
/// # Panics
///
/// This function does not panic under normal operation.
pub async fn create_event_backup_end<P: AsRef<Path>>(
    config: &Configuration,
    path: P,
    id: &[u8],
    source: EventSource,
    hostname: &str,
    backup_id: Uuid,
    num: usize,
    shares: &[&str],
    status: EventStatus,
) -> Result<()> {
    let event = Event {
        id: id.to_vec(),
        event_type: EventType::Backup as i32,
        step: EventStep::End as i32,
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs(),
        source: source as i32,
        user: String::new(),
        error_messages: Vec::new(),
        status: status as i32,

        information: Some(Information::Backup(EventBackupInformation {
            hostname: hostname.to_string(),
            number: num as u64,
            share_path: shares.iter().map(|s| (*s).to_string()).collect(),
            id: backup_id.to_string(),
        })),
    };

    append_events(config, path, &[&event]).await?;

    Ok(())
}

/// Logs the removal of a backup as an event.
///
/// # Arguments
///
/// * `config` - The application configuration.
/// * `path` - The directory where event files are stored.
/// * `source` - Source of the event (CLI, daemon, etc.).
/// * `hostname` - Hostname for which the backup is removed.
/// * `num` - Backup number.
/// * `shares` - List of share paths involved in the backup.
///
/// # Returns
///
/// * `Ok(())` if the event was logged successfully.
/// * `Err(eyre::Report)` if writing fails.
///
/// # Errors
///
/// Returns an error if writing fails.
///
/// # Panics
///
/// This function does not panic under normal operation.
pub async fn create_event_backup_remove<P: AsRef<Path>>(
    config: &Configuration,
    path: P,
    source: EventSource,
    hostname: &str,
    backup_id: Uuid,
    num: usize,
    shares: &[&str],
) -> Result<()> {
    let id = Uuid::new_v4();
    let id = id.as_bytes();

    let event = Event {
        id: id.to_vec(),
        event_type: EventType::Delete as i32,
        step: EventStep::Start as i32,
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs(),
        source: source as i32,
        user: String::new(),
        error_messages: Vec::new(),
        // Single-shot event, no End counterpart ever written: `Delete` is inherently
        // terminal, so there's no "in progress" state to distinguish from "done". The
        // display layer (frontend) treats this type as always-complete regardless of
        // status, rather than this field trying to encode a status that doesn't apply.
        status: EventStatus::None as i32,

        information: Some(Information::Backup(EventBackupInformation {
            hostname: hostname.to_string(),
            number: num as u64,
            share_path: shares.iter().map(|s| (*s).to_string()).collect(),
            id: backup_id.to_string(),
        })),
    };

    append_events(config, path, &[&event]).await?;

    Ok(())
}

impl Event {
    /// Serializes the event to a YAML string.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` if the event is successfully serialized.
    /// * `Err(eyre::Report)` if serialization fails.
    ///
    /// # Errors
    ///
    /// Returns an error if the event cannot be serialized to YAML.
    pub fn to_yaml(&self) -> Result<String> {
        let object = vec![self];
        let str = serde_yaml_ng::to_string(&object)?;
        Ok(str)
    }
}

impl fmt::Display for Event {
    /// Formats the event as YAML for display.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let yaml = self.to_yaml();
        let yaml = match yaml {
            Ok(yaml) => yaml,
            Err(err) => {
                return write!(f, "Failed to serialize FileManifest: {err}");
            }
        };

        // Write the formatted path to the Formatter
        write!(f, "{yaml}")
    }
}

use std::fmt::{self};
use std::{path::Path, time::SystemTime};

use chrono::NaiveDate;
use eyre::Result;
use tokio::fs;
use uuid::Uuid;

use crate::{
    proto::{ProtobufReader, ProtobufWriter, UnCompressedWriter},
    utils::lock::PoolLock,
    woodstock::event::Information,
    Event, EventBackupInformation, EventSource, EventStatus, EventStep, EventType,
};

/// Add events to the end of the file
///
/// # Arguments
///
/// * `path` - The path to the file to load.
/// * `event` - The events to append.
///
/// # Returns
///
/// * `Result<()>` - The result of the operation.
pub async fn append_events<P: AsRef<Path>>(path: P, events: &[&Event]) -> Result<()> {
    let path = path.as_ref();
    let lockfilename = path.with_file_name("lock");

    // Create the directory if it does not exist
    fs::create_dir_all(path).await?;

    let _lock = PoolLock::new_with_filename(&lockfilename, "events")
        .lock()
        .await?;

    // Get the current date
    let current_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let daily_path = path.join(format!("{}.events", current_date));

    let mut writer = ProtobufWriter::<UnCompressedWriter, Event>::open(daily_path).await?;

    for event in events {
        writer.write(event).await?;
    }

    writer.flush().await?;

    Ok(())
}

fn list_date(start_date: &NaiveDate, end_date: &NaiveDate) -> Vec<String> {
    let mut dates = Vec::new();
    let mut current_date = *start_date;
    while current_date <= *end_date {
        let current_date_str = current_date.format("%Y-%m-%d").to_string();

        dates.push(current_date_str);
        current_date += chrono::Duration::days(1);
    }

    dates
}

pub async fn read_events<P: AsRef<Path>>(
    path: P,
    start_date: &NaiveDate,
    end_data: &NaiveDate,
) -> Result<Vec<Event>> {
    let lockfilename = path.as_ref().with_extension("lock");
    let _lock = PoolLock::new_with_filename(&lockfilename, "events")
        .lock()
        .await?;

    let dates = list_date(start_date, end_data);

    let mut events = Vec::new();

    for date in dates {
        let daily_path = path.as_ref().join(format!("{}.events", date));
        if !daily_path.exists() {
            continue;
        }

        let mut reader = ProtobufReader::<Event>::new(daily_path, false).await?;
        reader.read_to_end(&mut events).await?;
    }

    Ok(events)
}

pub async fn create_event_backup_start<P: AsRef<Path>>(
    path: P,
    uuid: &[u8],
    source: EventSource,
    hostname: &str,
    num: usize,
    shares: &[&str],
) -> Result<()> {
    let event = Event {
        id: uuid.to_vec(),
        r#type: EventType::Backup as i32,
        step: EventStep::Start as i32,
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs(),
        source: source as i32,
        user: None,
        error_messages: Vec::new(),
        status: EventStatus::None as i32,

        information: Some(Information::Backup(EventBackupInformation {
            hostname: hostname.to_string(),
            number: num as u64,
            share_path: shares.iter().map(|s| s.to_string()).collect(),
        })),
    };

    append_events(path, &[&event]).await?;

    Ok(())
}

pub async fn create_event_backup_end<P: AsRef<Path>>(
    path: P,
    id: &[u8],
    source: EventSource,
    hostname: &str,
    num: usize,
    shares: &[&str],
    status: EventStatus,
) -> Result<()> {
    let event = Event {
        id: id.to_vec(),
        r#type: EventType::Backup as i32,
        step: EventStep::End as i32,
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs(),
        source: source as i32,
        user: None,
        error_messages: Vec::new(),
        status: status as i32,

        information: Some(Information::Backup(EventBackupInformation {
            hostname: hostname.to_string(),
            number: num as u64,
            share_path: shares.iter().map(|s| s.to_string()).collect(),
        })),
    };

    append_events(path, &[&event]).await?;

    Ok(())
}

pub async fn create_event_backup_remove<P: AsRef<Path>>(
    path: P,
    source: EventSource,
    hostname: &str,
    num: usize,
    shares: &[&str],
) -> Result<()> {
    let id = Uuid::new_v4();
    let id = id.as_bytes();

    let event = Event {
        id: id.to_vec(),
        r#type: EventType::Delete as i32,
        step: EventStep::Start as i32,
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs(),
        source: source as i32,
        user: None,
        error_messages: Vec::new(),
        status: EventStatus::None as i32,

        information: Some(Information::Backup(EventBackupInformation {
            hostname: hostname.to_string(),
            number: num as u64,
            share_path: shares.iter().map(|s| s.to_string()).collect(),
        })),
    };

    append_events(path, &[&event]).await?;

    Ok(())
}

impl Event {
    pub fn to_yaml(&self) -> Result<String> {
        let object = vec![self];
        let str = serde_yaml::to_string(&object)?;
        Ok(str)
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let yaml = self.to_yaml();
        let yaml = match yaml {
            Ok(yaml) => yaml,
            Err(err) => {
                return write!(f, "Failed to serialize FileManifest: {err}");
            }
        };

        // Écrivez le chemin formaté dans le Formatter
        write!(f, "{yaml}")
    }
}

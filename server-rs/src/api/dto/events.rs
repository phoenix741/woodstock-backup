use async_graphql::{Enum, SimpleObject, Union};
use chrono::{TimeZone, Utc};

use crate::graphql::scalars::BigIntScalar;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(remote = "woodstock::EventType")]
pub enum EventType {
    Backup,
    Restore,
    Delete,
    PoolChecked,
    PoolCleaned,
    HashConversion,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(remote = "woodstock::EventStep")]
pub enum EventStep {
    Start,
    End,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(remote = "woodstock::EventSource")]
pub enum EventSource {
    User,
    Woodstock,
    Import,
    Cli,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(remote = "woodstock::EventStatus")]
pub enum EventStatus {
    None,
    Success,
    ClientDisconnected,
    ServerCrashed,
    GenericError,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(remote = "woodstock::ChunkAlgorithm")]
pub enum ChunkAlgorithm {
    Blake3,
    #[graphql(name = "Sha3_256")]
    Sha3256,
    #[graphql(name = "Sha2_256")]
    Sha2256,
}

#[derive(SimpleObject, Clone)]
pub struct EventBackupInformation {
    pub hostname: String,
    pub number: i32,
    #[graphql(name = "sharePath")]
    pub share_path: Vec<String>,
}

#[derive(SimpleObject, Clone)]
pub struct EventPoolInformation {
    pub fix: bool,
    pub refcount: i32,
    #[graphql(name = "refcountError")]
    pub refcount_error: i32,
    #[graphql(name = "inUnused")]
    pub in_unused: i32,
    #[graphql(name = "inRefcnt")]
    pub in_refcnt: i32,
    #[graphql(name = "inNothing")]
    pub in_nothing: i32,
    pub missing: i32,
    #[graphql(name = "chunkCount")]
    pub chunk_count: i32,
    #[graphql(name = "chunkError")]
    pub chunk_error: i32,
}

#[derive(SimpleObject, Clone)]
pub struct EventPoolCleanedInformation {
    pub size: BigIntScalar,
    pub count: i32,
    #[graphql(name = "removedHashes")]
    pub removed_hashes: Vec<String>,
}

#[derive(SimpleObject, Clone)]
pub struct EventHashConversionInformation {
    pub count: i32,
    pub algorithm: ChunkAlgorithm,
}

#[derive(Union, Clone)]
pub enum EventInformation {
    EventBackupInformation(EventBackupInformation),
    EventPoolInformation(EventPoolInformation),
    EventPoolCleanedInformation(EventPoolCleanedInformation),
    EventHashConversionInformation(EventHashConversionInformation),
}

#[derive(SimpleObject, Clone)]
pub struct ApplicationEvent {
    pub uuid: String,
    #[graphql(name = "type")]
    pub type_: EventType,
    pub step: EventStep,
    pub source: EventSource,
    pub timestamp: chrono::DateTime<Utc>,
    #[graphql(name = "errorMessages")]
    pub error_messages: Vec<String>,
    pub status: EventStatus,
    pub information: Option<EventInformation>,
}

impl From<woodstock::Event> for ApplicationEvent {
    fn from(e: woodstock::Event) -> Self {
        let uuid =
            e.id.iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>();
        let ts = e.timestamp as i64;
        let timestamp = Utc
            .timestamp_opt(ts, 0)
            .single()
            // SAFETY: timestamp 0 is always valid for from_timestamp_millis()
            .unwrap_or(chrono::DateTime::<Utc>::UNIX_EPOCH);

        let information = e.information.and_then(|info| match info {
            woodstock::event::Information::Backup(b) => Some(
                EventInformation::EventBackupInformation(EventBackupInformation {
                    hostname: b.hostname,
                    number: b.number as i32,
                    share_path: b.share_path,
                }),
            ),
            woodstock::event::Information::PoolCleaned(pc) => Some(
                EventInformation::EventPoolCleanedInformation(EventPoolCleanedInformation {
                    size: BigIntScalar(pc.size),
                    count: pc.count as i32,
                    removed_hashes: pc
                        .removed_hashes
                        .iter()
                        .map(|h| h.iter().map(|b| format!("{:02x}", b)).collect::<String>())
                        .collect(),
                }),
            ),
            woodstock::event::Information::Pool(p) => Some(EventInformation::EventPoolInformation(
                EventPoolInformation {
                    fix: p.fix,
                    refcount: p.refcount as i32,
                    refcount_error: p.refcount_error as i32,
                    in_unused: p.in_unused as i32,
                    in_refcnt: p.in_refcnt as i32,
                    in_nothing: p.in_nothing as i32,
                    missing: p.missing as i32,
                    chunk_count: p.chunk_count as i32,
                    chunk_error: p.chunk_error as i32,
                },
            )),
            woodstock::event::Information::HashConversion(h) => Some(
                EventInformation::EventHashConversionInformation(EventHashConversionInformation {
                    count: h.count as i32,
                    algorithm: woodstock::ChunkAlgorithm::try_from(h.algorithm)
                        .unwrap_or(woodstock::ChunkAlgorithm::Blake3)
                        .into(),
                }),
            ),
        });

        Self {
            uuid,
            type_: (woodstock::EventType::try_from(e.event_type)
                .unwrap_or(woodstock::EventType::Backup))
            .into(),
            step: (woodstock::EventStep::try_from(e.step).unwrap_or(woodstock::EventStep::Start))
                .into(),
            source: (woodstock::EventSource::try_from(e.source)
                .unwrap_or(woodstock::EventSource::User))
            .into(),
            timestamp,
            error_messages: e.error_messages,
            status: (woodstock::EventStatus::try_from(e.status)
                .unwrap_or(woodstock::EventStatus::None))
            .into(),
            information,
        }
    }
}

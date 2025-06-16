//! Event model definitions for the Woodstock backup system.
//!
//! This module contains the data structures and enums representing events and their types
//! used throughout the Woodstock backup and restore process.

use napi::{bindgen_prelude::BigInt, Env, JsObject};
use uuid::Uuid;
use woodstock::{
  EventBackupInformation, EventPoolCleanedInformation, EventPoolInformation, EventSource,
  EventStatus, EventStep, EventType, HashConversionInformation,
};

use crate::config::configuration::JsChunkAlgorithm;

#[napi(string_enum)]
pub enum JsEventType {
  Backup,
  Restore,
  Delete,
  PoolChecked,
  PoolCleaned,
  HashConversion,
}

impl From<EventType> for JsEventType {
  fn from(event_type: EventType) -> Self {
    match event_type {
      EventType::Backup => JsEventType::Backup,
      EventType::Restore => JsEventType::Restore,
      EventType::Delete => JsEventType::Delete,
      EventType::PoolChecked => JsEventType::PoolChecked,
      EventType::PoolCleaned => JsEventType::PoolCleaned,
      EventType::HashConversion => JsEventType::HashConversion,
    }
  }
}

#[napi(string_enum)]
pub enum JsEventStep {
  Start,
  End,
}

impl From<EventStep> for JsEventStep {
  fn from(event_step: EventStep) -> Self {
    match event_step {
      EventStep::Start => JsEventStep::Start,
      EventStep::End => JsEventStep::End,
    }
  }
}

#[napi(string_enum)]
pub enum JsEventSource {
  User,
  Woodstock,
  Import,
  Cli,
}

impl From<EventSource> for JsEventSource {
  fn from(event_source: EventSource) -> Self {
    match event_source {
      EventSource::User => JsEventSource::User,
      EventSource::Woodstock => JsEventSource::Woodstock,
      EventSource::Import => JsEventSource::Import,
      EventSource::Cli => JsEventSource::Cli,
    }
  }
}

impl From<JsEventSource> for EventSource {
  fn from(event_source: JsEventSource) -> Self {
    match event_source {
      JsEventSource::User => EventSource::User,
      JsEventSource::Woodstock => EventSource::Woodstock,
      JsEventSource::Import => EventSource::Import,
      JsEventSource::Cli => EventSource::Cli,
    }
  }
}

#[napi(string_enum)]
pub enum JsEventStatus {
  None,
  Success,
  ClientDisconnected,
  ServerCrashed,
  GenericError,
}

impl From<EventStatus> for JsEventStatus {
  fn from(status: EventStatus) -> Self {
    match status {
      EventStatus::None => JsEventStatus::None,
      EventStatus::Success => JsEventStatus::Success,
      EventStatus::ClientDisconnected => JsEventStatus::ClientDisconnected,
      EventStatus::ServerCrashed => JsEventStatus::ServerCrashed,
      EventStatus::GenericError => JsEventStatus::GenericError,
    }
  }
}

#[napi(object)]
pub struct JsEventBackupInformation {
  pub hostname: String,
  pub number: u32,
  pub share_path: Vec<String>,
}

impl From<EventBackupInformation> for JsEventBackupInformation {
  fn from(event_information: EventBackupInformation) -> Self {
    JsEventBackupInformation {
      hostname: event_information.hostname,
      number: u32::try_from(event_information.number).unwrap(),
      share_path: event_information.share_path,
    }
  }
}

#[napi(object)]
pub struct JsEventPoolInformation {
  pub fix: bool,
  pub refcount: u32,
  pub refcount_error: u32,
  pub in_unused: u32,
  pub in_refcnt: u32,
  pub in_nothing: u32,
  pub missing: u32,
  pub chunk_count: u32,
  pub chunk_error: u32,
}

impl From<EventPoolInformation> for JsEventPoolInformation {
  fn from(event_information: EventPoolInformation) -> Self {
    JsEventPoolInformation {
      fix: event_information.fix,
      refcount: u32::try_from(event_information.refcount).unwrap(),
      refcount_error: u32::try_from(event_information.refcount_error).unwrap(),
      in_unused: u32::try_from(event_information.in_unused).unwrap(),
      in_refcnt: u32::try_from(event_information.in_refcnt).unwrap(),
      in_nothing: u32::try_from(event_information.in_nothing).unwrap(),
      missing: u32::try_from(event_information.missing).unwrap(),
      chunk_count: u32::try_from(event_information.chunk_count).unwrap(),
      chunk_error: u32::try_from(event_information.chunk_error).unwrap(),
    }
  }
}

#[napi(object)]
pub struct JsEventPoolCleanedInformation {
  pub size: BigInt,
  pub count: u32,
}

impl From<EventPoolCleanedInformation> for JsEventPoolCleanedInformation {
  fn from(event_information: EventPoolCleanedInformation) -> Self {
    JsEventPoolCleanedInformation {
      count: u32::try_from(event_information.count).unwrap(),
      size: event_information.size.into(),
    }
  }
}

#[napi(object)]
pub struct JsHashConversionInformation {
  pub count: u32,
  pub algorithm: JsChunkAlgorithm,
}

impl From<HashConversionInformation> for JsHashConversionInformation {
  fn from(event_information: HashConversionInformation) -> Self {
    JsHashConversionInformation {
      count: u32::try_from(event_information.count).unwrap(),
      algorithm: event_information.algorithm().into(),
    }
  }
}

#[napi(object)]
pub struct JsEvent {
  pub uuid: String,
  pub r#type: JsEventType,
  pub step: JsEventStep,
  pub timestamp: BigInt,
  pub source: JsEventSource,
  pub user: String,
  pub error_messages: Vec<String>,
  pub status: JsEventStatus,
  #[napi(
    ts_type = "JsEventBackupInformation | JsEventPoolInformation | JsEventPoolCleanedInformation | JsHashConversionInformation"
  )]
  pub information: Option<JsObject>,
}

impl JsEvent {
  #[must_use]
  /// Create a `JsEvent` from a core Woodstock event and a N-API environment.
  ///
  /// # Panics
  /// Panics if the event UUID is not 16 bytes long and cannot be converted.
  pub fn from_js(event: woodstock::Event, env: Env) -> Self {
    let uuid = &event.id;
    let uuid = if uuid.len() == 16 {
      Uuid::from_bytes(uuid.as_slice().try_into().expect("UUID must be 16 bytes")).to_string()
    } else {
      Uuid::new_v4().to_string()
    };

    let information = match &event.information {
      Some(woodstock::event::Information::Backup(info)) => {
        let mut obj = env.create_object().expect("Failed to create object");
        obj.set("hostname", info.hostname.clone()).unwrap();
        obj
          .set("number", u32::try_from(info.number).unwrap())
          .unwrap();
        obj.set("sharePath", info.share_path.clone()).unwrap();
        Some(obj)
      }
      Some(woodstock::event::Information::Pool(info)) => {
        let mut obj = env.create_object().expect("Failed to create object");
        obj.set("fix", info.fix).unwrap();
        obj
          .set("refcount", u32::try_from(info.refcount).unwrap())
          .unwrap();
        obj
          .set("refcountError", u32::try_from(info.refcount_error).unwrap())
          .unwrap();
        obj
          .set("inUnused", u32::try_from(info.in_unused).unwrap())
          .unwrap();
        obj
          .set("inRefcnt", u32::try_from(info.in_refcnt).unwrap())
          .unwrap();
        obj
          .set("inNothing", u32::try_from(info.in_nothing).unwrap())
          .unwrap();
        obj
          .set("missing", u32::try_from(info.missing).unwrap())
          .unwrap();
        obj
          .set("chunkCount", u32::try_from(info.chunk_count).unwrap())
          .unwrap();
        obj
          .set("chunkError", u32::try_from(info.chunk_error).unwrap())
          .unwrap();
        Some(obj)
      }
      Some(woodstock::event::Information::PoolCleaned(info)) => {
        let mut obj = env.create_object().expect("Failed to create object");
        obj.set("size", info.size).unwrap();
        obj
          .set("count", u32::try_from(info.count).unwrap())
          .unwrap();
        Some(obj)
      }
      Some(woodstock::event::Information::HashConversion(info)) => {
        let mut obj = env.create_object().expect("Failed to create object");
        obj
          .set("count", u32::try_from(info.count).unwrap())
          .unwrap();
        obj
          .set("algorithm", info.algorithm().as_str_name())
          .unwrap();
        Some(obj)
      }
      None => None,
    };

    JsEvent {
      uuid,
      r#type: event.r#type().into(),
      step: event.step().into(),
      timestamp: event.timestamp.into(),
      source: event.source().into(),
      status: event.status().into(),
      user: event.user,
      error_messages: event.error_messages,
      information,
    }
  }
}

use napi::{bindgen_prelude::BigInt, Env, JsObject};
use uuid::Uuid;
use woodstock::{
  EventBackupInformation, EventPoolCleanedInformation, EventPoolInformation,
  EventRefCountInformation, EventSource, EventStatus, EventStep, EventType,
};

#[napi]
pub enum JsEventType {
  Backup,
  Restore,
  Delete,
  RefcntChecked,
  PoolChecked,
  ChecksumChecked,
  PoolCleaned,
}

impl From<EventType> for JsEventType {
  fn from(event_type: EventType) -> Self {
    match event_type {
      EventType::Backup => JsEventType::Backup,
      EventType::Restore => JsEventType::Restore,
      EventType::Delete => JsEventType::Delete,
      EventType::RefcntChecked => JsEventType::RefcntChecked,
      EventType::PoolChecked => JsEventType::PoolChecked,
      EventType::ChecksumChecked => JsEventType::ChecksumChecked,
      EventType::PoolCleaned => JsEventType::PoolCleaned,
    }
  }
}

#[napi]
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

#[napi]
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

#[napi]
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
  pub number: BigInt,
  pub share_path: Vec<String>,
}

impl From<EventBackupInformation> for JsEventBackupInformation {
  fn from(event_information: EventBackupInformation) -> Self {
    JsEventBackupInformation {
      hostname: event_information.hostname,
      number: event_information.number.into(),
      share_path: event_information.share_path,
    }
  }
}

#[napi(object)]
pub struct JsEventRefCountInformation {
  pub fix: bool,
  pub count: BigInt,
  pub error: BigInt,
}

impl From<EventRefCountInformation> for JsEventRefCountInformation {
  fn from(event_information: EventRefCountInformation) -> Self {
    JsEventRefCountInformation {
      fix: event_information.fix,
      count: event_information.count.into(),
      error: event_information.error.into(),
    }
  }
}

#[napi(object)]
pub struct JsEventPoolInformation {
  pub fix: bool,
  pub in_unused: BigInt,
  pub in_refcnt: BigInt,
  pub in_nothing: BigInt,
  pub missing: BigInt,
}

impl From<EventPoolInformation> for JsEventPoolInformation {
  fn from(event_information: EventPoolInformation) -> Self {
    JsEventPoolInformation {
      fix: event_information.fix,
      in_unused: event_information.in_unused.into(),
      in_refcnt: event_information.in_refcnt.into(),
      in_nothing: event_information.in_nothing.into(),
      missing: event_information.missing.into(),
    }
  }
}

#[napi(object)]
pub struct JsEventPoolCleanedInformation {
  pub size: BigInt,
  pub count: BigInt,
}

impl From<EventPoolCleanedInformation> for JsEventPoolCleanedInformation {
  fn from(event_information: EventPoolCleanedInformation) -> Self {
    JsEventPoolCleanedInformation {
      count: event_information.count.into(),
      size: event_information.size.into(),
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
  pub user: Option<String>,
  pub error_messages: Vec<String>,
  pub status: JsEventStatus,
  #[napi(
    ts_type = "JsEventBackupInformation | JsEventRefCountInformation | JsEventPoolInformation | JsEventPoolCleanedInformation"
  )]
  pub information: Option<JsObject>,
}

impl JsEvent {
  pub fn from_js(event: woodstock::Event, env: Env) -> Self {
    let uuid = &event.id;
    let uuid = if uuid.len() != 16 {
      Uuid::new_v4().to_string()
    } else {
      Uuid::from_bytes(uuid.as_slice().try_into().expect("UUID must be 16 bytes")).to_string()
    };

    let information = match &event.information {
      Some(woodstock::event::Information::Backup(info)) => {
        let mut obj = env.create_object().expect("Failed to create object");
        obj.set("hostname", info.hostname.clone()).unwrap();
        obj.set("number", info.number).unwrap();
        obj.set("sharePath", info.share_path.clone()).unwrap();
        Some(obj)
      }
      Some(woodstock::event::Information::Refcnt(info)) => {
        let mut obj = env.create_object().expect("Failed to create object");
        obj.set("fix", info.fix).unwrap();
        obj.set("count", info.count).unwrap();
        obj.set("error", info.error).unwrap();
        Some(obj)
      }
      Some(woodstock::event::Information::Pool(info)) => {
        let mut obj = env.create_object().expect("Failed to create object");
        obj.set("fix", info.fix).unwrap();
        obj.set("inUnused", info.in_unused).unwrap();
        obj.set("inRefcnt", info.in_refcnt).unwrap();
        obj.set("inNothing", info.in_nothing).unwrap();
        obj.set("missing", info.missing).unwrap();
        Some(obj)
      }
      Some(woodstock::event::Information::PoolCleaned(info)) => {
        let mut obj = env.create_object().expect("Failed to create object");
        obj.set("size", info.size).unwrap();
        obj.set("count", info.count).unwrap();
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

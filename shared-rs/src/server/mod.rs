//! Server module for Woodstock shared library.
//!
//! This module provides server-side services and utilities for the Woodstock backup system.
//! It includes backup, restore, pool management, and related server operations.
mod abort_handle;
// pub mod remove;
pub mod resolve;
// pub mod restore;
pub mod backup_remove_service;
pub mod backup_restore_service;
pub mod backup_save_service;
pub mod pool_cleaner_service;
pub mod pool_fsck_service;
pub mod tools;

pub use abort_handle::AbortHandle;

use std::time::SystemTime;

use napi::bindgen_prelude::BigInt;
use woodstock::server::progression::BackupProgression;

use crate::config::configuration::JsLogLevel;

#[napi(object)]
pub struct JsBackupProgression {
  pub start_date: i64,
  pub start_transfer_date: Option<i64>,
  pub end_transfer_date: Option<i64>,

  pub compressed_file_size: BigInt,
  pub new_compressed_file_size: BigInt,
  pub modified_compressed_file_size: BigInt,

  pub file_size: BigInt,
  pub new_file_size: BigInt,
  pub modified_file_size: BigInt,

  pub new_file_count: u32,
  pub file_count: u32,
  pub modified_file_count: u32,
  pub removed_file_count: u32,

  pub error_count: u32,

  pub progress_current: BigInt,
  pub progress_max: BigInt,

  pub percent: f64,
  pub speed: f64,
}

impl From<&BackupProgression> for JsBackupProgression {
  fn from(progression: &BackupProgression) -> Self {
    Self {
      start_date: i64::try_from(
        progression
          .start_date
          .duration_since(SystemTime::UNIX_EPOCH)
          .unwrap()
          .as_secs(),
      )
      .unwrap(),
      start_transfer_date: progression.start_transfer_date.map(|date| {
        i64::try_from(
          date
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        )
        .unwrap()
      }),
      end_transfer_date: progression.end_transfer_date.map(|date| {
        i64::try_from(
          date
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        )
        .unwrap()
      }),

      compressed_file_size: BigInt::from(progression.compressed_file_size),
      new_compressed_file_size: BigInt::from(progression.new_compressed_file_size),
      modified_compressed_file_size: BigInt::from(progression.modified_compressed_file_size),

      file_size: BigInt::from(progression.file_size),
      new_file_size: BigInt::from(progression.new_file_size),
      modified_file_size: BigInt::from(progression.modified_file_size),

      new_file_count: progression.new_file_count as u32,
      file_count: progression.file_count as u32,
      modified_file_count: progression.modified_file_count as u32,
      removed_file_count: progression.removed_file_count as u32,

      error_count: progression.error_count as u32,

      progress_current: BigInt::from(progression.progress_current),
      progress_max: BigInt::from(progression.progress_max),

      percent: progression.percent(),
      speed: progression.speed(),
    }
  }
}

#[napi(object)]
pub struct JsLogEntry {
  pub level: Option<JsLogLevel>,
  pub context: Option<String>,
  pub line: Option<String>,
}

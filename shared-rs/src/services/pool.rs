use napi::{
  bindgen_prelude::BigInt,
  threadsafe_function::{ErrorStrategy, ThreadsafeFunction},
  Error, JsFunction, Result,
};
use woodstock::{
  config::Context,
  pool::{add_refcnt_to_pool, remove_refcnt_to_pool, FsckCount, FsckUnusedCount},
  server::pool_fsck::{FsckProgression, PoolFsck, PoolProgression},
  EventPoolCleanedInformation, EventPoolInformation, EventRefCountInformation,
};

use crate::{config::context::JsBackupContext, server::AbortHandle};

#[napi(object)]
pub struct JsPoolProgression {
  pub progress_current: u32,

  pub file_count: u32,
  pub file_size: BigInt,
  pub compressed_file_size: BigInt,
}

impl From<PoolProgression> for JsPoolProgression {
  fn from(unused: PoolProgression) -> Self {
    Self {
      progress_current: u32::try_from(unused.progress_current).unwrap_or_default(),
      file_count: u32::try_from(unused.file_count).unwrap_or_default(),
      file_size: BigInt::from(unused.file_size),
      compressed_file_size: BigInt::from(unused.compressed_file_size),
    }
  }
}

#[napi(object)]
pub struct JsFsckProgression {
  pub error_count: u32,
  pub total_count: u32,

  pub progress_current: u32,
}

impl From<FsckProgression> for JsFsckProgression {
  fn from(p: FsckProgression) -> Self {
    Self {
      error_count: u32::try_from(p.error_count).unwrap_or_default(),
      total_count: u32::try_from(p.total_count).unwrap_or_default(),
      progress_current: u32::try_from(p.progress_current).unwrap_or_default(),
    }
  }
}

#[napi(object)]
pub struct JsEventPoolCleanedInformation {
  pub size: BigInt,
  pub count: BigInt,
}

impl From<EventPoolCleanedInformation> for JsEventPoolCleanedInformation {
  fn from(event: EventPoolCleanedInformation) -> Self {
    Self {
      size: BigInt::from(event.size),
      count: BigInt::from(event.count),
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
  fn from(event: EventRefCountInformation) -> Self {
    Self {
      fix: event.fix,
      count: BigInt::from(event.count),
      error: BigInt::from(event.error),
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
  fn from(event: EventPoolInformation) -> Self {
    Self {
      fix: event.fix,
      in_unused: BigInt::from(event.in_unused),
      in_refcnt: BigInt::from(event.in_refcnt),
      in_nothing: BigInt::from(event.in_nothing),
      missing: BigInt::from(event.missing),
    }
  }
}

#[napi(object)]
pub struct CleanedUnusedMessage {
  pub progress: Option<JsPoolProgression>,
  pub error: Option<String>,
  pub complete: Option<JsEventPoolCleanedInformation>,
}

#[napi(object)]
pub struct FsckProgressMessage {
  pub progress: Option<JsFsckProgression>,
  pub error: Option<String>,
  pub complete: Option<JsEventRefCountInformation>,
}

#[napi(object)]
pub struct FsckUnusedMessage {
  pub progress: Option<JsPoolProgression>,
  pub error: Option<String>,
  pub complete: Option<JsEventPoolInformation>,
}

#[napi(object)]
pub struct JsFsckCount {
  pub error_count: BigInt,
  pub total_count: BigInt,
}

impl From<FsckCount> for JsFsckCount {
  fn from(fsck_count: FsckCount) -> Self {
    Self {
      error_count: BigInt::from(u64::try_from(fsck_count.error_count).unwrap_or_default()),
      total_count: BigInt::from(u64::try_from(fsck_count.total_count).unwrap_or_default()),
    }
  }
}

#[napi(object)]
pub struct JsFsckUnusedCount {
  pub in_unused: BigInt,
  pub in_refcnt: BigInt,
  pub in_nothing: BigInt,
  pub missing: BigInt,
}

impl From<FsckUnusedCount> for JsFsckUnusedCount {
  fn from(fsck_count: FsckUnusedCount) -> Self {
    Self {
      in_unused: BigInt::from(u64::try_from(fsck_count.in_unused).unwrap_or_default()),
      in_refcnt: BigInt::from(u64::try_from(fsck_count.in_refcnt).unwrap_or_default()),
      in_nothing: BigInt::from(u64::try_from(fsck_count.in_nothing).unwrap_or_default()),
      missing: BigInt::from(u64::try_from(fsck_count.missing).unwrap_or_default()),
    }
  }
}

#[napi(js_name = "CorePoolService")]
pub struct JsPoolService {
  context: Context,
  fsck: PoolFsck,
}

#[napi]
impl JsPoolService {
  #[napi(constructor)]
  #[must_use]
  pub fn new(context: &JsBackupContext) -> Self {
    let context: Context = context.into();

    Self {
      context: context.clone(),
      fsck: PoolFsck::new(&context),
    }
  }

  #[napi]
  pub async fn add_refcnt_of_pool(&self, hostname: String, backup_number: u32) -> Result<()> {
    let backup_number = usize::try_from(backup_number)
      .map_err(|_| Error::from_reason("Backup number is too large".to_string()))?;

    add_refcnt_to_pool(&self.context, &hostname, backup_number)
      .await
      .map_err(|_| Error::from_reason("Failed to remove refcnt of host".to_string()))?;

    Ok(())
  }

  #[napi]
  pub async fn remove_refcnt_of_pool(&self, hostname: String, backup_number: u32) -> Result<()> {
    let backup_number = usize::try_from(backup_number)
      .map_err(|_| Error::from_reason("Backup number is too large".to_string()))?;

    remove_refcnt_to_pool(&self.context, &hostname, backup_number)
      .await
      .map_err(|_| Error::from_reason("Failed to remove refcnt of host".to_string()))?;

    Ok(())
  }

  #[napi]
  pub async fn remove_unused_max(&self) -> Result<u32> {
    let max = self
      .fsck
      .clean_unused_max()
      .await
      .map_err(|_| Error::from_reason("Failed to list unused file".to_string()))?;
    let max = u32::try_from(max).unwrap_or_default();

    Ok(max)
  }

  #[napi]
  pub fn remove_unused(
    &self,
    target: Option<String>,
    #[napi(ts_arg_type = "(result: CleanedUnusedMessage) => void")] callback: JsFunction,
  ) -> Result<AbortHandle> {
    let tsfn: ThreadsafeFunction<CleanedUnusedMessage, ErrorStrategy::Fatal> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

    let target = target.map(std::convert::Into::into);
    let fsck = self.fsck.clone();

    let handle = tokio::spawn(async move {
      let result = fsck
        .clean_unused_pool(target, &|unused| {
          tsfn.call(
            CleanedUnusedMessage {
              progress: Some(unused.clone().into()),
              error: None,
              complete: None,
            },
            napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
          );
        })
        .await
        .map_err(|_| Error::from_reason("Failed to remove unused files".to_string()));

      match result {
        Ok(information) => {
          tsfn.call(
            CleanedUnusedMessage {
              progress: None,
              error: None,
              complete: Some(information.into()),
            },
            napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
          );
        }
        Err(e) => {
          tsfn.call(
            CleanedUnusedMessage {
              progress: None,
              error: Some(e.to_string()),
              complete: None,
            },
            napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
          );
        }
      }
    });

    Ok(AbortHandle::new(handle))
  }

  #[napi]
  pub async fn verify_chunk_max(&self) -> Result<u32> {
    let max = self
      .fsck
      .verify_chunk_max()
      .await
      .map_err(|_| Error::from_reason("Failed to list chunk file".to_string()))?
      .len();
    let max = u32::try_from(max).unwrap_or_default();

    Ok(max)
  }

  #[napi]
  pub fn verify_chunk(
    &self,
    #[napi(ts_arg_type = "(result: FsckProgressMessage) => void")] callback: JsFunction,
  ) -> Result<AbortHandle> {
    let tsfn: ThreadsafeFunction<FsckProgressMessage, ErrorStrategy::Fatal> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

    let fsck = self.fsck.clone();
    let handle = tokio::spawn(async move {
      let result = fsck
        .verify_chunk(&|progression| {
          tsfn.call(
            FsckProgressMessage {
              progress: Some(progression.clone().into()),
              error: None,
              complete: None,
            },
            napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
          );
        })
        .await
        .map_err(|_| Error::from_reason("Failed to remove unused files".to_string()));

      match result {
        Ok(information) => {
          tsfn.call(
            FsckProgressMessage {
              progress: None,
              error: None,
              complete: Some(information.into()),
            },
            napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
          );
        }
        Err(e) => {
          tsfn.call(
            FsckProgressMessage {
              progress: None,
              error: Some(e.to_string()),
              complete: None,
            },
            napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
          );
        }
      };
    });

    Ok(AbortHandle::new(handle))
  }

  #[napi]
  pub async fn verify_refcnt_max(&self) -> Result<u32> {
    let max = self
      .fsck
      .verify_refcnt_max()
      .await
      .map_err(|_| Error::from_reason("Failed to list refcnt file".to_string()))?;
    let max = u32::try_from(max).unwrap_or_default();

    Ok(max)
  }

  #[napi]
  pub fn verify_refcnt(
    &self,
    dry_run: bool,
    #[napi(ts_arg_type = "(result: FsckProgressMessage) => void")] callback: JsFunction,
  ) -> Result<AbortHandle> {
    let tsfn: ThreadsafeFunction<FsckProgressMessage, ErrorStrategy::Fatal> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

    let fsck = self.fsck.clone();
    let handle = tokio::spawn(async move {
      let result = fsck
        .verify_refcnt(dry_run, &|progression| {
          tsfn.call(
            FsckProgressMessage {
              progress: Some(progression.clone().into()),
              error: None,
              complete: None,
            },
            napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
          );
        })
        .await
        .map_err(|_| Error::from_reason("Failed to remove unused files".to_string()));

      match result {
        Ok(information) => {
          tsfn.call(
            FsckProgressMessage {
              progress: None,
              error: None,
              complete: Some(information.into()),
            },
            napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
          );
        }
        Err(e) => {
          tsfn.call(
            FsckProgressMessage {
              progress: None,
              error: Some(e.to_string()),
              complete: None,
            },
            napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
          );
        }
      };
    });

    Ok(AbortHandle::new(handle))
  }

  #[napi]
  pub async fn verify_unused_max(&self) -> Result<u32> {
    let max = self
      .fsck
      .verify_unused_max()
      .await
      .map_err(|_| Error::from_reason("Failed to list unused file".to_string()))?;
    let max = u32::try_from(max).unwrap_or_default();

    Ok(max)
  }

  #[napi]
  pub fn verify_unused(
    &self,
    dry_run: bool,
    #[napi(ts_arg_type = "(result: FsckUnusedMessage) => void")] callback: JsFunction,
  ) -> Result<AbortHandle> {
    let tsfn: ThreadsafeFunction<FsckUnusedMessage, ErrorStrategy::Fatal> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

    let fsck = self.fsck.clone();
    let handle = tokio::spawn(async move {
      let result = fsck
        .verify_unused(dry_run, &|progression| {
          tsfn.call(
            FsckUnusedMessage {
              progress: Some(progression.clone().into()),
              error: None,
              complete: None,
            },
            napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
          );
        })
        .await
        .map_err(|_| Error::from_reason("Failed to remove unused files".to_string()));

      match result {
        Ok(information) => {
          tsfn.call(
            FsckUnusedMessage {
              progress: None,
              error: None,
              complete: Some(information.into()),
            },
            napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
          );
        }
        Err(e) => {
          tsfn.call(
            FsckUnusedMessage {
              progress: None,
              error: Some(e.to_string()),
              complete: None,
            },
            napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
          );
        }
      };
    });

    Ok(AbortHandle::new(handle))
  }
}

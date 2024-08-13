use std::sync::atomic::AtomicUsize;

use napi::{
  bindgen_prelude::BigInt,
  threadsafe_function::{ErrorStrategy, ThreadsafeFunction},
  Error, JsFunction, Result,
};
use woodstock::{
  config::Context,
  pool::{
    add_refcnt_to_pool, check_backup_integrity, check_host_integrity, check_pool_integrity,
    check_unused, remove_refcnt_to_pool, FsckCount, FsckUnusedCount, PoolChunkWrapper, Refcnt,
  },
  PoolUnused,
};

use crate::{config::context::JsBackupContext, server::AbortHandle};

#[napi(object)]
pub struct JsPoolUnused {
  pub sha256: Vec<u8>,
  pub size: BigInt,
  pub compressed_size: BigInt,
}

impl From<PoolUnused> for JsPoolUnused {
  fn from(unused: PoolUnused) -> Self {
    Self {
      sha256: unused.sha256.clone(),
      size: unused.size.into(),
      compressed_size: unused.compressed_size.into(),
    }
  }
}

#[napi(object)]
pub struct PoolUnusedMessage {
  pub progress: Option<JsPoolUnused>,
  pub error: Option<String>,
  pub complete: bool,
}

#[napi(object)]
pub struct VerifyChunkCount {
  pub total_count: BigInt,
  pub error_count: BigInt,
}

#[napi(object)]
pub struct VerifyChunkMessage {
  pub progress: Option<VerifyChunkCount>,
  pub error: Option<String>,
  pub complete: bool,
}

#[napi(js_name = "CorePoolService")]
pub struct JsPoolService {
  context: Context,
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
pub struct VerifyChunkProgress {
  pub total_count: BigInt,
  pub progress_count: BigInt,
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

#[napi(object)]
pub struct JsFsckUnusedCountMessage {
  pub progress: Option<VerifyChunkProgress>,
  pub error: Option<String>,
  pub complete: Option<JsFsckUnusedCount>,
}

#[napi]
impl JsPoolService {
  #[napi(constructor)]
  #[must_use]
  pub fn new(context: &JsBackupContext) -> Self {
    let context: Context = context.into();

    Self {
      context: context.clone(),
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
  pub async fn count_unused(&self) -> Result<u64> {
    let pool_path = self.context.config.path.pool_path.clone();

    let mut refcnt = Refcnt::new(&pool_path);
    refcnt.load_unused().await;

    let unused_count = refcnt.list_unused().count();
    let unused_count = u64::try_from(unused_count).unwrap_or_default();

    Ok(unused_count)
  }

  #[napi]
  pub fn remove_unused(
    &self,
    target: Option<String>,
    #[napi(ts_arg_type = "(result: PoolUnusedMessage) => void")] callback: JsFunction,
  ) -> Result<AbortHandle> {
    let tsfn: ThreadsafeFunction<PoolUnusedMessage, ErrorStrategy::Fatal> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

    let pool_path = self.context.config.path.pool_path.clone();
    let target = target.map(std::convert::Into::into);

    let handle = tokio::spawn(async move {
      let mut refcnt = Refcnt::new(&pool_path);
      refcnt.load_unused().await;

      let result = refcnt
        .remove_unused_files(&pool_path, target, &|unused| {
          tsfn.call(
            PoolUnusedMessage {
              progress: unused.clone().map(std::convert::Into::into),
              error: None,
              complete: false,
            },
            napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
          );
        })
        .await
        .map_err(|_| Error::from_reason("Failed to remove unused files".to_string()));

      match result {
        Ok(()) => {
          tsfn.call(
            PoolUnusedMessage {
              progress: None,
              error: None,
              complete: true,
            },
            napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
          );
        }
        Err(e) => {
          tsfn.call(
            PoolUnusedMessage {
              progress: None,
              error: Some(e.to_string()),
              complete: true,
            },
            napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
          );
        }
      }
    });

    Ok(AbortHandle::new(handle))
  }

  #[napi]
  pub async fn count_chunk(&self) -> Result<u64> {
    let mut pool_refcnt = Refcnt::new(&self.context.config.path.pool_path);
    pool_refcnt.load_refcnt(false).await;
    pool_refcnt.load_unused().await;

    let mut chunks = pool_refcnt
      .list_refcnt()
      .map(|refcnt| refcnt.sha256.clone())
      .collect::<Vec<_>>();
    chunks.extend(
      pool_refcnt
        .list_unused()
        .map(|unused| unused.sha256.clone()),
    );

    let chunks_len = u64::try_from(chunks.len()).unwrap_or_default();

    Ok(chunks_len)
  }

  #[napi]
  pub fn verify_chunk(
    &self,
    #[napi(ts_arg_type = "(result: VerifyChunkMessage) => void")] callback: JsFunction,
  ) -> Result<AbortHandle> {
    let tsfn: ThreadsafeFunction<VerifyChunkMessage, ErrorStrategy::Fatal> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let pool_path = self.context.config.path.pool_path.clone();

    let handle = tokio::spawn(async move {
      let mut pool_refcnt = Refcnt::new(&pool_path);
      pool_refcnt.load_refcnt(false).await;
      pool_refcnt.load_unused().await;

      let mut chunks = pool_refcnt
        .list_refcnt()
        .map(|refcnt| refcnt.sha256.clone())
        .collect::<Vec<_>>();
      chunks.extend(
        pool_refcnt
          .list_unused()
          .map(|unused| unused.sha256.clone()),
      );

      let mut error_count: usize = 0;
      let mut total_count: usize = 0;

      for refcnt in chunks {
        let wrapper = PoolChunkWrapper::new(&pool_path, Some(&refcnt));

        let is_valid = wrapper.check_chunk_information().await.unwrap_or(false);
        if !is_valid {
          error_count += 1;
        }

        total_count += 1;

        tsfn.call(
          VerifyChunkMessage {
            progress: Some(VerifyChunkCount {
              total_count: BigInt::from(u64::try_from(total_count).unwrap_or_default()),
              error_count: BigInt::from(u64::try_from(error_count).unwrap_or_default()),
            }),
            error: None,
            complete: false,
          },
          napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
        );
      }

      tsfn.call(
        VerifyChunkMessage {
          progress: None,
          error: None,
          complete: true,
        },
        napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
      );
    });

    Ok(AbortHandle::new(handle))
  }

  #[napi]
  pub async fn check_backup_integrity(
    &self,
    hostname: String,
    backup_number: u32,
    dry_run: bool,
  ) -> Result<JsFsckCount> {
    let backup_number = usize::try_from(backup_number)
      .map_err(|_| Error::from_reason("Backup number is too large".to_string()))?;

    let result = check_backup_integrity(&hostname, backup_number, dry_run, &self.context)
      .await
      .map_err(|_| Error::from_reason("Can't check backup integrity"))?;

    Ok(result.into())
  }

  #[napi]
  pub async fn check_host_integrity(&self, hostname: String, dry_run: bool) -> Result<JsFsckCount> {
    let result = check_host_integrity(&hostname, dry_run, &self.context)
      .await
      .map_err(|_| Error::from_reason("Can't check host integrity"))?;

    Ok(result.into())
  }

  #[napi]
  pub async fn check_pool_integrity(&self, dry_run: bool) -> Result<JsFsckCount> {
    let result = check_pool_integrity(dry_run, &self.context)
      .await
      .map_err(|_| Error::from_reason("Can't check pool integrity"))?;

    Ok(result.into())
  }

  // #[napi]
  // pub async fn check_unused(&self, dry_run: bool) -> Result<JsFsckUnusedCount> {
  //   let result = check_unused(dry_run, &|_| {}, &self.context)
  //     .await
  //     .map_err(|_| Error::from_reason("Can't check pool integrity"))?;

  //   Ok(result.into())
  // }

  #[napi]
  pub fn check_unused(
    &self,
    dry_run: bool,
    #[napi(ts_arg_type = "(result: JsFsckUnusedCountMessage) => void")] callback: JsFunction,
  ) -> Result<AbortHandle> {
    let tsfn: ThreadsafeFunction<JsFsckUnusedCountMessage, ErrorStrategy::Fatal> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let progress_tsfn = tsfn.clone();
    let context = self.context.clone();

    let handle = tokio::spawn(async move {
      let mut pool_refcnt = Refcnt::new(&context.config.path.pool_path);
      pool_refcnt.load_refcnt(false).await;
      pool_refcnt.load_unused().await;

      let total = pool_refcnt.list_unused().count() + pool_refcnt.list_refcnt().count();
      let count = AtomicUsize::new(0);

      let result = check_unused(
        dry_run,
        &move |p| {
          count.fetch_add(p, std::sync::atomic::Ordering::AcqRel);
          progress_tsfn.call(
            JsFsckUnusedCountMessage {
              progress: Some(VerifyChunkProgress {
                total_count: BigInt::from(u64::try_from(total).unwrap_or_default()),
                progress_count: BigInt::from(
                  u64::try_from(count.load(std::sync::atomic::Ordering::Acquire))
                    .unwrap_or_default(),
                ),
              }),
              error: None,
              complete: None,
            },
            napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
          );
        },
        &context,
      )
      .await;

      match result {
        Ok(result) => {
          tsfn.call(
            JsFsckUnusedCountMessage {
              progress: Some(VerifyChunkProgress {
                total_count: BigInt::from(u64::try_from(total).unwrap_or_default()),
                progress_count: BigInt::from(u64::try_from(total).unwrap_or_default()),
              }),
              error: None,
              complete: Some(result.into()),
            },
            napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
          );
        }
        Err(e) => {
          tsfn.call(
            JsFsckUnusedCountMessage {
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
}

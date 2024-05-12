use std::{collections::HashMap, path::PathBuf};

use futures_util::{pin_mut, StreamExt};
use napi::{
  bindgen_prelude::Buffer,
  threadsafe_function::{
    ErrorStrategy::{self},
    ThreadsafeFunction, ThreadsafeFunctionCallMode,
  },
  Error, JsFunction, Result,
};
use tokio::io::AsyncReadExt;
use woodstock::{
  config::{Backups, Context, BUFFER_SIZE},
  utils::path::vec_to_path,
  FileManifest,
};

use crate::{config::context::JsBackupContext, models::JsFileManifest};

#[napi(js_name = "CoreFilesService")]
pub struct JsFilesService {
  backups: Backups,
  pool_path: PathBuf,
}

#[napi]
impl JsFilesService {
  #[napi(constructor)]
  #[must_use]
  pub fn new(context: &JsBackupContext) -> Self {
    let context: Context = context.into();

    Self {
      backups: Backups::new(&context),
      pool_path: context.config.path.pool_path.clone(),
    }
  }

  #[napi]
  pub fn list(
    &self,
    hostname: String,
    backup_number: u32,
    share_path: String,
    path: Buffer,
    recursive: bool,

    #[napi(ts_arg_type = "(err: null | Error, result: JsFileManifest | null) => void")]
    callback: JsFunction,
  ) -> Result<()> {
    let tsfn: ThreadsafeFunction<Option<JsFileManifest>, ErrorStrategy::CalleeHandled> =
      callback.create_threadsafe_function(0, |ctx| {
        let Some(value) = ctx.value else {
          return Ok(vec![]);
        };
        Ok(vec![value])
      })?;

    let backup_number = usize::try_from(backup_number)
      .map_err(|_| Error::from_reason("Backup number is too large".to_string()))?;
    let manifest = self
      .backups
      .get_manifest(&hostname, backup_number, &share_path);

    let path: Vec<u8> = path.into();
    let search_path = vec_to_path(&path);

    let tsfn = tsfn.clone();

    tokio::spawn(async move {
      let entries = manifest.read_manifest_entries();
      pin_mut!(entries);

      while let Some(entry) = entries.next().await {
        let path = entry.path();
        // If not recursif we keep only all files that is in the search path directory, else we keep all file that are in a lower level directory
        if (recursive && path.starts_with(&search_path)) || (!recursive && path == search_path) {
          let file = JsFileManifest::from(entry);

          tsfn.call(Ok(Some(file)), ThreadsafeFunctionCallMode::Blocking);
        }
      }

      tsfn.call(Ok(None), ThreadsafeFunctionCallMode::Blocking);
    });

    Ok(())
  }

  #[napi]
  pub fn read_manifest(
    &self,
    manifest: JsFileManifest,

    #[napi(ts_arg_type = "(err: null | Error, result: Buffer) => void")] callback: JsFunction,
  ) -> Result<()> {
    let tsfn: ThreadsafeFunction<Option<Vec<u8>>, ErrorStrategy::CalleeHandled> = callback
      .create_threadsafe_function(0, |ctx| {
        let Some(value) = ctx.value else {
          return Ok(vec![]);
        };
        Ok(vec![value])
      })?;

    let manifest: FileManifest = FileManifest {
      path: manifest.path.into(),
      hash: manifest.hash.into(),
      chunks: manifest.chunks.iter().map(|c| c.clone().into()).collect(),
      stats: None,
      symlink: Vec::new(),
      xattr: Vec::new(),
      acl: Vec::new(),
      metadata: HashMap::new(),
    };
    let pool_path = self.pool_path.clone();

    tokio::spawn(async move {
      let reader = manifest.open_from_pool(&pool_path);
      pin_mut!(reader);

      let mut buffer = vec![0; BUFFER_SIZE];

      loop {
        let Ok(size) = reader.read(&mut buffer).await else {
          tsfn.call(
            Err(Error::from_reason(
              "Error while reading the manifest".to_string(),
            )),
            ThreadsafeFunctionCallMode::Blocking,
          );
          break;
        };
        if size == 0 {
          break;
        }

        tsfn.call(
          Ok(Some(buffer[..size].to_vec())),
          ThreadsafeFunctionCallMode::Blocking,
        );
      }

      tsfn.call(Ok(None), ThreadsafeFunctionCallMode::Blocking);
    });

    Ok(())
  }
}

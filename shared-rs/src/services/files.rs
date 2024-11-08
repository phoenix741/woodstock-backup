use std::{collections::HashMap, path::PathBuf, sync::Arc};

use futures_util::pin_mut;
use napi::{
  bindgen_prelude::Buffer,
  threadsafe_function::{
    ErrorStrategy::{self},
    ThreadsafeFunction, ThreadsafeFunctionCallMode,
  },
  Error, JsFunction, Result,
};
use tokio::{io::AsyncReadExt, sync::Mutex};
use woodstock::{
  config::{Context, BUFFER_SIZE},
  utils::path::vec_to_path,
  view::WoodstockView,
  FileManifest,
};

use crate::{config::context::JsBackupContext, models::JsFileManifest};

#[napi(js_name = "ViewerService")]
pub struct JsViewerService {
  view: Arc<Mutex<WoodstockView>>,
  hostname: String,
  backup_number: usize,
}

#[napi]
impl JsViewerService {
  #[must_use]
  #[napi(constructor)]
  pub fn new(context: &JsBackupContext, hostname: String, backup_number: u32) -> Self {
    let context: Context = context.into();
    let backup_number = usize::try_from(backup_number).expect("Backup number is too large");

    Self {
      view: Arc::new(Mutex::new(WoodstockView::new(&context))),
      hostname,
      backup_number,
    }
  }

  #[napi]
  pub async fn list_dir(&self, share_path: String, path: Buffer) -> Result<Vec<JsFileManifest>> {
    let path: Vec<u8> = path.into();
    let path = vec_to_path(&path);
    let path = path.strip_prefix("/").unwrap_or(&path).to_path_buf();

    let mut view = self.view.lock().await;

    let entries = view
      .list_file_from_dir(&self.hostname, self.backup_number, &share_path, &path)
      .await
      .map_err(|err| Error::from_reason(err.to_string()))?;

    let entries = entries.iter().cloned().map(JsFileManifest::from).collect();

    Ok(entries)
  }

  #[napi]
  pub async fn list_dir_recursive(
    &self,
    share_path: String,
    path: Buffer,
  ) -> Result<Vec<JsFileManifest>> {
    let path: Vec<u8> = path.into();
    let path = vec_to_path(&path);
    let path = path.strip_prefix("/").unwrap_or(&path).to_path_buf();

    let mut view = self.view.lock().await;

    let entries = view
      .list_all_files(&self.hostname, self.backup_number, &share_path, &path)
      .await
      .map_err(|err| Error::from_reason(err.to_string()))?;

    let entries = entries.iter().cloned().map(JsFileManifest::from).collect();

    Ok(entries)
  }
}

#[napi(js_name = "CoreFilesService")]
pub struct JsFilesService {
  context: JsBackupContext,
  pool_path: PathBuf,
}

#[napi]
impl JsFilesService {
  #[napi(constructor)]
  #[must_use]
  pub fn new(context: &JsBackupContext) -> Self {
    let rust_context: Context = context.into();
    let pool_path = rust_context.config.path.pool_path.clone();

    let context = (*context).clone();

    Self { context, pool_path }
  }

  #[napi]
  pub fn create_viewer(&self, hostname: String, backup_number: u32) -> Result<JsViewerService> {
    Ok(JsViewerService::new(&self.context, hostname, backup_number))
  }

  #[napi]
  pub fn read_file(
    &self,
    manifest: JsFileManifest,

    #[napi(ts_arg_type = "(err: null | Error, result: Buffer) => void")] callback: JsFunction,
  ) -> Result<()> {
    let tsfn: ThreadsafeFunction<Option<Buffer>, ErrorStrategy::CalleeHandled> = callback
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
          Ok(Some(Buffer::from(buffer[..size].to_vec()))),
          ThreadsafeFunctionCallMode::Blocking,
        );
      }

      tsfn.call(Ok(None), ThreadsafeFunctionCallMode::Blocking);
    });

    Ok(())
  }
}

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
  config::{GlobalConfiguration, BUFFER_SIZE},
  utils::{path::vec_to_path, thread::spawn_with_context},
  view::WoodstockView,
  FileManifest,
};

use crate::models::JsFileManifest;

#[napi(js_name = "ViewerService")]
/// Provides file viewing capabilities for a specific host and backup number.
///
/// This struct manages the Woodstock view and allows listing and reading files for a backup.
///
/// # Fields
/// * `view` - The Woodstock view instance, protected by a mutex for async access.
/// * `hostname` - The hostname associated with the backup.
/// * `backup_number` - The backup number.
pub struct JsViewerService {
  /// The Woodstock view instance, protected by a mutex for async access.
  view: Arc<Mutex<WoodstockView>>,
  /// The hostname associated with the backup.
  hostname: String,
  /// The backup number.
  backup_number: usize,
}

#[napi]
impl JsViewerService {
  #[must_use]
  #[napi(constructor)]
  /// Creates a new `JsViewerService` for the specified host and backup number.
  ///
  /// # Arguments
  /// * `hostname` - The hostname associated with the backup.
  /// * `backup_number` - The backup number.
  ///
  /// # Panics
  /// Panics if the backup number cannot be converted to `usize`.
  pub fn new(hostname: String, backup_number: u32) -> Self {
    let backup_number = usize::try_from(backup_number).expect("Backup number is too large");

    Self {
      view: Arc::new(Mutex::new(WoodstockView::new(&GlobalConfiguration))),
      hostname,
      backup_number,
    }
  }

  #[napi]
  /// Lists the files in the specified directory for the given share and path.
  ///
  /// # Arguments
  /// * `share_path` - The share path to list files from.
  /// * `path` - The directory path as a buffer.
  ///
  /// # Errors
  /// Returns an error if the directory cannot be listed.
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
  /// Recursively lists all files in the specified directory for the given share and path.
  ///
  /// # Arguments
  /// * `share_path` - The share path to list files from.
  /// * `path` - The directory path as a buffer.
  ///
  /// # Errors
  /// Returns an error if the directory cannot be listed.
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
/// Provides file management services for the Woodstock backup system.
///
/// This struct manages the pool path and allows creating file viewers and reading files from the backup pool.
///
/// # Fields
/// * `pool_path` - The path to the backup pool.
pub struct JsFilesService {
  /// The path to the backup pool.
  pool_path: PathBuf,
}

impl Default for JsFilesService {
  fn default() -> Self {
    Self::new()
  }
}

#[napi]
impl JsFilesService {
  #[napi(constructor)]
  #[must_use]
  /// Creates a new `JsFilesService` instance.
  ///
  /// # Returns
  /// A new instance of `JsFilesService` with the default pool path.
  pub fn new() -> Self {
    let pool_path = GlobalConfiguration.path.pool_path.clone();

    Self { pool_path }
  }

  #[napi]
  /// Creates a new file viewer for the specified host and backup number.
  ///
  /// # Arguments
  /// * `hostname` - The hostname associated with the backup.
  /// * `backup_number` - The backup number.
  ///
  /// # Errors
  /// Returns an error if the viewer cannot be created.
  pub fn create_viewer(&self, hostname: String, backup_number: u32) -> Result<JsViewerService> {
    Ok(JsViewerService::new(hostname, backup_number))
  }

  #[napi]
  /// Reads a file from the backup pool and invokes the callback with the file contents.
  ///
  /// # Arguments
  /// * `manifest` - The file manifest describing the file to read.
  /// * `callback` - A JavaScript callback function to receive the file contents as a buffer.
  ///
  /// # Errors
  /// Returns an error if the file cannot be read or if the callback cannot be invoked.
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

    spawn_with_context(async move {
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

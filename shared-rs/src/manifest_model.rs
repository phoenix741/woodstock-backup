use std::path::Path;

#[napi(js_name = "Manifest")]
pub struct Manifest {
  pub manifest_path: String,
  pub file_list_path: String,
  pub journal_path: String,
  pub new_path: String,
  pub lock_path: String,
}

#[napi]
impl Manifest {
  #[napi(constructor)]
  pub fn new(manifest_name: String, path: String) -> Self {
    let path = Path::new(&path);
    Self {
      manifest_path: path
        .join(format!("{}.manifest", manifest_name))
        .into_os_string()
        .into_string()
        .unwrap(),
      file_list_path: path
        .join(format!("{}.filelist", manifest_name))
        .into_os_string()
        .into_string()
        .unwrap(),
      journal_path: path
        .join(format!("{}.journal", manifest_name))
        .into_os_string()
        .into_string()
        .unwrap(),
      new_path: path
        .join(format!("{}.new", manifest_name))
        .into_os_string()
        .into_string()
        .unwrap(),
      lock_path: path
        .join(format!("/{}.lock", manifest_name))
        .into_os_string()
        .into_string()
        .unwrap(),
    }
  }
}

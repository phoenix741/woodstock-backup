use std::collections::HashMap;

use crate::woodstock::{EntryType, FileManifest, FileManifestJournalEntry};

pub struct IndexFileEntry {
  pub mark_viewed: bool,
  pub manifest: FileManifest,
}

impl IndexFileEntry {
  pub fn path(&self) -> &Vec<u8> {
    &self.manifest.path
  }
}

pub struct IndexManifest {
  files: HashMap<Vec<u8>, IndexFileEntry>,
}

impl IndexManifest {
  pub fn new() -> Self {
    Self {
      files: HashMap::new(),
    }
  }

  pub fn apply(&mut self, journal_entry: FileManifestJournalEntry) {
    if journal_entry.r#type() == EntryType::Remove {
      if let Some(manifest) = journal_entry.manifest {
        self.remove(&manifest.path);
      }
    } else {
      if let Some(manifest) = journal_entry.manifest {
        self.add(manifest);
      }
    }
  }

  pub fn walk(&mut self) -> impl Iterator<Item = &IndexFileEntry> {
    self.files.values()
  }

  pub fn indexSize(&self) -> usize {
    self.files.len()
  }

  pub fn add(&mut self, manifest: FileManifest) {
    let key = manifest.path.clone();
    self.files.insert(
      key,
      IndexFileEntry {
        mark_viewed: false,
        manifest,
      },
    );
  }

  pub fn remove(&mut self, file_path: &Vec<u8>) {
    self.files.remove(file_path);
  }

  pub fn mark(&mut self, file_path: &Vec<u8>) {
    if let Some(entry) = self.files.get_mut(file_path) {
      entry.mark_viewed = true;
    }
  }

  pub fn getEntry(&self, file_path: &Vec<u8>) -> Option<&IndexFileEntry> {
    self.files.get(file_path)
  }
}

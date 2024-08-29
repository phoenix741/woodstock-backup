use std::path::Path;

use async_stream::stream;
use console::{style, Term};
use eyre::Result;
use futures::{pin_mut, Stream, StreamExt};
use indicatif::HumanBytes;
use woodstock::{
    manifest::{Manifest, PathManifest},
    EntryState, EntryType, FileManifestJournalEntry,
};

pub fn generate_compare_stream(
    manifest1: &str,
    manifest2: &str,
) -> impl Stream<Item = FileManifestJournalEntry> {
    let manifest1 = Path::new(manifest1);
    let manifest1 = Manifest::new(
        manifest1
            .with_extension("")
            .file_name()
            .unwrap()
            .to_str()
            .unwrap(),
        manifest1.parent().unwrap_or_else(|| Path::new("")),
    );

    let manifest2 = Path::new(manifest2);
    let manifest2 = Manifest::new(
        manifest2
            .with_extension("")
            .file_name()
            .unwrap()
            .to_str()
            .unwrap(),
        manifest2.parent().unwrap_or_else(|| Path::new("")),
    );

    stream!({
        let mut index = manifest1.load_index().await;
        let stream_manifest_2 = manifest2.read_manifest_entries();
        pin_mut!(stream_manifest_2);

        while let Some(manifest) = stream_manifest_2.next().await {
            let entry = index.mark(&manifest.path);
            if let Some(entry) = entry {
                if entry.manifest.hash.ne(&manifest.hash) {
                    yield FileManifestJournalEntry {
                        manifest: Some(manifest),
                        r#type: EntryType::Modify as i32,

                        state: EntryState::Todo as i32,
                        state_message: None,
                    };
                }
            } else {
                yield FileManifestJournalEntry {
                    manifest: Some(manifest),
                    r#type: EntryType::Add as i32,

                    state: EntryState::Todo as i32,
                    state_message: None,
                };
            }
        }

        let remove_stream = index.walk();
        pin_mut!(remove_stream);

        for entry in remove_stream.by_ref() {
            if !entry.mark_viewed {
                yield FileManifestJournalEntry {
                    manifest: Some(entry.manifest.clone()),
                    r#type: EntryType::Remove as i32,

                    state: EntryState::Todo as i32,
                    state_message: None,
                };
            }
        }
    })
}

pub async fn compare(manifest1: &str, manifest2: &str) -> Result<()> {
    let term = Term::stdout();

    let stream = generate_compare_stream(manifest1, manifest2);
    pin_mut!(stream);

    while let Some(entry) = stream.next().await {
        let entry_type = entry.r#type();

        let Some(manifest) = entry.manifest else {
            term.write_line("entry without path")?;
            continue;
        };

        let path = manifest.path();
        let size = HumanBytes(manifest.size());

        match entry_type {
            EntryType::Add => {
                term.write_line(&style(format!("+{path:?} {size}")).green().to_string())?;
            }
            EntryType::Modify => {
                term.write_line(&style(format!("*{path:?} {size}")).yellow().to_string())?;
            }
            EntryType::Remove => {
                term.write_line(&style(format!("-{path:?} {size}")).red().to_string())?;
            }
        }
    }
    Ok(())
}

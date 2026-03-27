use std::path::{Path, PathBuf};

use eyre::{bail, eyre, Result};
use tokio::fs::{create_dir_all, metadata, File};
use tokio::io::{AsyncWriteExt, BufReader};

use super::segment_protobuf::{SegmentFileMetadataRecord, SegmentHeader};
use crate::proto::{read_length_delimited_message, write_length_delimited_message};

use super::segment_model::{SegmentFileHeader, SegmentFileMetadata};

#[derive(Debug)]
pub(crate) struct SegmentOpenMetadata {
    pub path: PathBuf,
    pub header: SegmentFileHeader,
    pub size_total: u64,
    pub data_offset: u64,
}

#[must_use]
pub(crate) fn segment_sidecar_metadata_path(path: &Path) -> PathBuf {
    let mut path = path.as_os_str().to_os_string();
    path.push(".meta");
    PathBuf::from(path)
}

pub(crate) async fn open_segment_metadata<P: AsRef<Path>>(path: P) -> Result<SegmentOpenMetadata> {
    let path = path.as_ref().to_path_buf();
    let file = File::open(&path).await?;
    let mut file = BufReader::new(file);
    let Some((header_record, header_size)) =
        read_length_delimited_message::<_, SegmentHeader>(&mut file).await?
    else {
        bail!("segment file is empty or missing header");
    };
    let file_size = metadata(&path).await?.len();
    let data_offset = u64::try_from(header_size)?;

    Ok(SegmentOpenMetadata {
        path,
        header: header_record.into(),
        size_total: file_size,
        data_offset,
    })
}

pub(crate) async fn read_persisted_segment_file_metadata<P: AsRef<Path>>(
    path: P,
) -> Result<SegmentFileMetadata> {
    let metadata_path: PathBuf = segment_sidecar_metadata_path(path.as_ref());
    let file = File::open(&metadata_path).await?;
    let mut file = BufReader::new(file);
    let Some((record, _)) =
        read_length_delimited_message::<_, SegmentFileMetadataRecord>(&mut file).await?
    else {
        bail!("segment metadata file is empty or missing record");
    };

    Ok(SegmentFileMetadata::try_from(record).map_err(|e| eyre!("Can't read metadata {e}"))?)
}

pub(crate) async fn write_segment_file_metadata<P: AsRef<Path>>(
    path: P,
    metadata: &SegmentFileMetadata,
) -> Result<()> {
    let metadata_path = segment_sidecar_metadata_path(path.as_ref());
    if let Some(parent) = metadata_path.parent() {
        create_dir_all(parent).await?;
    }

    let mut file = File::create(&metadata_path).await?;
    let record = SegmentFileMetadataRecord::from(metadata);
    write_length_delimited_message(&mut file, &record).await?;
    file.flush().await?;
    file.shutdown().await?;
    Ok(())
}

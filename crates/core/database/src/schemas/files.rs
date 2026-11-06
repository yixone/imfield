use chrono::{DateTime, Utc};
use imfield_mimetype::{MimeKind, MimeType};
use imfield_models::types::{files::FileVariant, id::FileContainerId};
use imfield_storage_address::StoragePath;
use sqlx::prelude::FromRow;

pub type ContaineredFilesRow = (FileContainerRow, Vec<FileRow>);

#[derive(FromRow, Clone)]
pub struct FileContainerRow {
    pub id: FileContainerId,
    pub original_checksum: Vec<u8>,
    pub original_filename: Option<String>,
    pub original_mimekind: MimeKind,
}

#[derive(FromRow, Clone)]
pub struct FileRow {
    pub container_id: FileContainerId,
    pub variant: FileVariant,
    pub created_at: DateTime<Utc>,
    pub path: StoragePath,
    pub size_bytes: u64,
    pub mimetype: MimeType,
    pub duration_ms: Option<i64>,
}

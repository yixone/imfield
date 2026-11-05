use chrono::{DateTime, Utc};
use imfield_mimetype::MimeType;
use imfield_models::types::{files::FileVariant, id::FileContainerId};
use imfield_storage_address::StoragePath;
use sqlx::prelude::FromRow;

#[derive(FromRow, Clone)]
pub struct FileContainerRow {
    pub id: FileContainerId,
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

pub struct FileContainerRows {
    pub container: FileContainerRow,
    pub files: Vec<FileRow>,
}

coalesced!(FileRow[container_id] => FileContainerRow[id] as FileContainerId);
coalesced!(FileContainerRow[id] => FileRow[container_id] as FileContainerId);

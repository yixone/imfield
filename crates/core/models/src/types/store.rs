use std::time::Duration;

use chrono::{DateTime, Utc};
use imfield_mimetype::{MimeKind, MimeType};
use imfield_storage_address::StoragePath;

use crate::{
    entities::asset::{AssetData, AssetState},
    types::{
        files::{FileVariant, Sha256Checksum},
        id::{AssetId, FileContainerId},
        media::{Color, PerceptualHash, ThumbHash},
    },
};

pub struct AssetStoreEntry {
    pub id: AssetId,
    pub file_container: FileContainerId,
    pub data: AssetData,
    pub accent_color: Option<Color>,
    pub dimension: Option<(u32, u32)>,
    pub thumbhash: Option<ThumbHash>,
    pub state: AssetState,
    pub is_offline: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub features_updated_at: Option<DateTime<Utc>>,
    pub p_hash: Option<PerceptualHash>,
    pub a_hash: Option<PerceptualHash>,
}

pub struct FileStoreEntry {
    pub container: FileContainerId,
    pub variant: FileVariant,
    pub created_at: DateTime<Utc>,
    pub path: StoragePath,
    pub size_bytes: u64,
    pub mimetype: MimeType,
    pub duration: Option<Duration>,
}

pub struct FileContainerStoreEntry {
    pub id: FileContainerId,
    pub original_checksum: Sha256Checksum,
    pub original_filename: Option<String>,
    pub original_mimekind: MimeKind,
}

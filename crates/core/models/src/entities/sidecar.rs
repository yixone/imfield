use std::time::Duration;

use chrono::{DateTime, Utc};
use imfield_mimetype::MimeType;
use imfield_storage_address::StoragePath;

use crate::types::id::FileId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum SidecarType {
    Thumbnail,
    LoopPreview,
}

impl SidecarType {
    /// Returns the [`SidecarType`] as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Thumbnail => "thumbnail",
            Self::LoopPreview => "loop_preview",
        }
    }
}

#[derive(Debug)]
pub struct Sidecar {
    /// ID of the sidecar source file
    pub(crate) file_id: FileId,

    /// Sidecar creation time
    pub(crate) created_at: DateTime<Utc>,

    /// Sidecar file path in application storage
    pub(crate) path: StoragePath,

    /// Sidecar file size in bytes
    pub(crate) size_bytes: u64,

    /// Sidecar MIME type
    pub(crate) mimetype: MimeType,

    /// Sidecar type
    pub(crate) sidecar_type: SidecarType,

    /// Sidecar file duration (for supported files)
    pub(crate) duration: Option<Duration>,
}

pub struct SidecarData {
    pub path: StoragePath,
    pub size_bytes: u64,
    pub mimetype: MimeType,
    pub duration: Option<Duration>,
}

pub struct SidecarDbEntry {
    pub file_id: FileId,
    pub sidecar_type: SidecarType,
    pub created_at: DateTime<Utc>,
    pub data: SidecarData,
}

impl Sidecar {
    pub fn new(file_id: FileId, sidecar_type: SidecarType, data: SidecarData) -> Self {
        Sidecar {
            file_id,
            created_at: Utc::now(),
            path: data.path,
            size_bytes: data.size_bytes,
            mimetype: data.mimetype,
            sidecar_type,
            duration: data.duration,
        }
    }

    pub fn from_db(entry: SidecarDbEntry) -> Self {
        Sidecar {
            file_id: entry.file_id,
            created_at: entry.created_at,
            path: entry.data.path,
            size_bytes: entry.data.size_bytes,
            mimetype: entry.data.mimetype,
            sidecar_type: entry.sidecar_type,
            duration: entry.data.duration,
        }
    }

    pub fn id(&self) -> &FileId {
        &self.file_id
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn path(&self) -> &StoragePath {
        &self.path
    }

    pub fn size_bytes(&self) -> u64 {
        self.size_bytes
    }

    pub fn mimetype(&self) -> MimeType {
        self.mimetype
    }

    pub fn sidecar_type(&self) -> &SidecarType {
        &self.sidecar_type
    }

    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }
}

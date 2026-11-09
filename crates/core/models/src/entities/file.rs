use std::{collections::HashSet, time::Duration};

use chrono::{DateTime, Utc};
use imfield_mimetype::{MimeKind, MimeType};
use imfield_storage_address::StoragePath;

use crate::{
    entities::{Sidecar, sidecar::SidecarType},
    types::{files::Sha256Checksum, id::FileId},
};

/// Virtual file resource, including all embedded data
#[derive(Debug, Clone)]
pub struct FileResource {
    pub file: File,
    pub sidecars: Vec<Sidecar>,
}

impl FileResource {
    pub fn sidecar_variants(&self) -> HashSet<SidecarType> {
        self.sidecars.iter().map(|s| s.sidecar_type()).collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum FileVariant {
    Original,
    #[cfg_attr(feature = "serde", serde(untagged))]
    Sidecar(SidecarType),
}

impl FileVariant {
    /// Returns the [`FileVariant`] as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Original => "original",
            Self::Sidecar(s) => s.as_str(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct File {
    /// Snowflake File id
    pub(crate) id: FileId,

    /// File creation time
    pub(crate) created_at: DateTime<Utc>,

    /// File path in application storage
    pub(crate) path: StoragePath,

    /// File size in bytes
    pub(crate) size_bytes: u64,

    /// File MIME type
    pub(crate) mimetype: MimeType,

    /// File duration (for supported files)
    pub(crate) duration: Option<Duration>,

    /// Original filename, if available
    pub(crate) original_filename: Option<String>,

    /// SHA-256 checksum of the original file, used for
    /// content verification
    pub(crate) checksum: Sha256Checksum,
}

pub struct FileData {
    pub path: StoragePath,
    pub size_bytes: u64,
    pub mimetype: MimeType,
    pub duration: Option<Duration>,
    pub original_filename: Option<String>,
    pub checksum: Sha256Checksum,
}

pub struct FileDbEntry {
    pub id: FileId,
    pub created_at: DateTime<Utc>,
    pub data: FileData,
}

impl File {
    pub fn new(id: FileId, data: FileData) -> Self {
        File {
            id,
            created_at: Utc::now(),
            path: data.path,
            size_bytes: data.size_bytes,
            mimetype: data.mimetype,
            duration: data.duration,
            original_filename: data.original_filename,
            checksum: data.checksum,
        }
    }

    pub fn from_db(entry: FileDbEntry) -> Self {
        File {
            id: entry.id,
            created_at: entry.created_at,
            path: entry.data.path,
            size_bytes: entry.data.size_bytes,
            mimetype: entry.data.mimetype,
            duration: entry.data.duration,
            original_filename: entry.data.original_filename,
            checksum: entry.data.checksum,
        }
    }

    pub fn to_db(self) -> FileDbEntry {
        FileDbEntry {
            id: self.id,
            created_at: self.created_at,
            data: FileData {
                path: self.path,
                size_bytes: self.size_bytes,
                mimetype: self.mimetype,
                duration: self.duration,
                original_filename: self.original_filename,
                checksum: self.checksum,
            },
        }
    }

    pub fn id(&self) -> &FileId {
        &self.id
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

    pub fn mimekind(&self) -> MimeKind {
        self.mimetype().kind()
    }

    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }

    pub fn set_duration(&mut self, duration: Option<Duration>) {
        self.duration = duration;
    }

    pub fn original_filename(&self) -> Option<&str> {
        self.original_filename.as_deref()
    }

    pub fn checksum(&self) -> Sha256Checksum {
        self.checksum
    }
}

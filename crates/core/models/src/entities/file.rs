use std::time::Duration;

use chrono::{DateTime, Utc};
use imfield_mimetype::{MimeKind, MimeType};
use imfield_storage_address::StoragePath;

use crate::types::{
    files::{FileVariant, Sha256Checksum},
    id::{FileContainerId, FileKey},
    store::{FileContainerStoreEntry, FileStoreEntry},
};

/// Virtual file container resource, including all embedded data
#[derive(Debug)]
pub struct FileContainerResource {
    /// File container model
    pub container: FileContainer,
    /// Container's files
    pub files: Vec<File>,
}

#[derive(Debug)]
pub struct FileContainer {
    /// File container id
    pub(crate) id: FileContainerId,

    /// SHA-256 checksum of the original file, used for
    /// content verification
    pub(crate) original_checksum: Sha256Checksum,
    /// Original filename, if available
    pub(crate) original_filename: Option<String>,
    /// MIME kind of the original file
    pub(crate) original_mimekind: MimeKind,
}

pub struct OriginalFileData {
    checksum: Sha256Checksum,
    filename: Option<String>,
    mimekind: MimeKind,
}

impl FileContainer {
    /// Creates a new [`FileContainer`]
    pub fn new(id: FileContainerId, original: OriginalFileData) -> FileContainer {
        FileContainer {
            id,
            original_checksum: original.checksum,
            original_filename: original.filename,
            original_mimekind: original.mimekind,
        }
    }

    pub fn from_stored(entry: FileContainerStoreEntry) -> Self {
        FileContainer {
            id: entry.id,
            original_checksum: entry.original_checksum,
            original_filename: entry.original_filename,
            original_mimekind: entry.original_mimekind,
        }
    }

    pub fn to_stored(self) -> FileContainerStoreEntry {
        FileContainerStoreEntry {
            id: self.id,
            original_checksum: self.original_checksum,
            original_filename: self.original_filename,
            original_mimekind: self.original_mimekind,
        }
    }

    /// Returns the file container's identifier
    pub fn id(&self) -> &FileContainerId {
        &self.id
    }

    /// Returns the SHA-256 checksum of the original file
    pub fn original_checksum(&self) -> Sha256Checksum {
        self.original_checksum
    }

    /// Returns the orignal filename, if available
    pub fn original_filename(&self) -> Option<&str> {
        self.original_filename.as_deref()
    }

    /// Returns the MIME kind of the original file
    pub fn original_mimekind(&self) -> MimeKind {
        self.original_mimekind
    }
}

#[derive(Debug)]
pub struct File {
    /// File key (FileContainerId + Variant)
    pub(crate) key: FileKey,

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
}

impl File {
    pub fn new(
        container: &FileContainerId,
        variant: FileVariant,
        path: StoragePath,
        size_bytes: u64,
        mimetype: MimeType,
        duration: Option<Duration>,
    ) -> Self {
        File {
            key: FileKey(container.clone(), variant),
            created_at: Utc::now(),
            path,
            size_bytes,
            mimetype,
            duration,
        }
    }

    pub fn from_stored(entry: FileStoreEntry) -> Self {
        let key = FileKey(entry.container, entry.variant);

        File {
            key,
            created_at: entry.created_at,
            path: entry.path,
            size_bytes: entry.size_bytes,
            mimetype: entry.mimetype,
            duration: entry.duration,
        }
    }

    pub fn to_stored(self) -> FileStoreEntry {
        FileStoreEntry {
            container: self.key.0,
            variant: self.key.1,
            created_at: self.created_at,
            path: self.path,
            size_bytes: self.size_bytes,
            mimetype: self.mimetype,
            duration: self.duration,
        }
    }

    pub fn key(&self) -> &FileKey {
        &self.key
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

    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }

    pub fn set_duration(&mut self, duration: Option<Duration>) {
        self.duration = duration;
    }
}

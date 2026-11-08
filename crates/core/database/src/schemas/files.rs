use std::{collections::HashMap, time::Duration};

use chrono::{DateTime, Utc};
use imfield_mimetype::{MimeKind, MimeType};
use imfield_models::{
    DomainError,
    entities::{File, FileContainer, file::FileResource},
    types::{
        files::{FileVariant, Sha256Checksum},
        id::FileContainerId,
        store::{FileContainerStoreEntry, FileStoreEntry},
    },
};
use imfield_storage_address::StoragePath;
use sqlx::prelude::FromRow;

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

/// Maps files database rows to domain entities and resource models
pub struct FileRowMapper;

impl FileRowMapper {
    pub fn to_container_domain(container: FileContainerRow) -> Result<FileContainer, DomainError> {
        let entry = FileContainerStoreEntry {
            id: container.id,
            original_checksum: Sha256Checksum::new(
                container
                    .original_checksum
                    .try_into()
                    .map_err(|_| DomainError::PersistenceError)?,
            ),
            original_filename: container.original_filename,
            original_mimekind: container.original_mimekind,
        };
        Ok(FileContainer::from_stored(entry))
    }

    pub fn to_container_domain_many(
        containers: Vec<FileContainerRow>,
    ) -> Result<Vec<FileContainer>, DomainError> {
        containers
            .into_iter()
            .map(FileRowMapper::to_container_domain)
            .collect()
    }

    pub fn to_file_domain(file: FileRow) -> Result<File, DomainError> {
        let entry = FileStoreEntry {
            container: file.container_id,
            variant: file.variant,
            created_at: file.created_at,
            path: file.path,
            size_bytes: file.size_bytes,
            mimetype: file.mimetype,
            duration: file.duration_ms.map(|ms| Duration::from_millis(ms as u64)),
        };
        Ok(File::from_stored(entry))
    }

    pub fn to_file_domain_many(files: Vec<FileRow>) -> Result<Vec<File>, DomainError> {
        files
            .into_iter()
            .map(FileRowMapper::to_file_domain)
            .collect()
    }

    pub fn to_resource(container: FileContainer, files: Vec<File>) -> FileResource {
        FileResource { container, files }
    }

    pub fn to_resource_many(containers: Vec<FileContainer>, files: Vec<File>) -> Vec<FileResource> {
        let mut index: HashMap<FileContainerId, Vec<File>> =
            HashMap::with_capacity(containers.len());
        for f in files {
            match index.get_mut(f.key().container()) {
                Some(v) => v.push(f),
                None => {
                    index.insert(f.key().container().clone(), vec![f]);
                }
            }
        }

        let mut resources = Vec::with_capacity(containers.len());
        for c in containers {
            let f = index.get(c.id()).cloned().unwrap_or(Vec::new());
            resources.push(FileResource {
                container: c,
                files: f,
            });
        }

        resources
    }
}

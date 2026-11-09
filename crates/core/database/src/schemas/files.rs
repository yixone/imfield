use std::{collections::HashMap, time::Duration};

use chrono::{DateTime, Utc};
use imfield_mimetype::{MimeKind, MimeType};
use imfield_models::{
    DomainError,
    entities::{
        File, Sidecar,
        file::{FileData, FileDbEntry, FileResource},
        sidecar::{SidecarData, SidecarDbEntry, SidecarType},
    },
    types::{files::Sha256Checksum, id::FileId},
};
use imfield_storage_address::StoragePath;
use sqlx::prelude::FromRow;

#[derive(FromRow, Clone)]
pub struct FileRow {
    pub id: FileId,
    pub created_at: DateTime<Utc>,
    pub path: StoragePath,
    pub size_bytes: i64,
    pub mimetype: MimeType,
    pub mimekind: MimeKind,
    pub duration_ms: Option<i64>,
    pub original_filename: Option<String>,
    pub checksum: Vec<u8>,
}

#[derive(FromRow, Clone)]
pub struct SidecarRow {
    pub file_id: FileId,
    pub sidecar_type: SidecarType,
    pub created_at: DateTime<Utc>,
    pub path: StoragePath,
    pub size_bytes: i64,
    pub mimetype: MimeType,
    pub duration_ms: Option<i64>,
}

/// Maps files database rows to domain entities and resource models
pub struct FileRowMapper;

impl FileRowMapper {
    pub fn to_file_domain(file: FileRow) -> Result<File, DomainError> {
        let entry = FileDbEntry {
            id: file.id,
            created_at: file.created_at,
            data: FileData {
                path: file.path,
                size_bytes: file.size_bytes as u64,
                mimetype: file.mimetype,
                mimekind: file.mimekind,
                duration: file.duration_ms.map(|t| Duration::from_millis(t as u64)),
                original_filename: file.original_filename,
                checksum: Sha256Checksum::new(
                    file.checksum
                        .try_into()
                        .map_err(|_| DomainError::PersistenceError)?,
                ),
            },
        };
        Ok(File::from_db(entry))
    }

    pub fn to_file_domain_many(files: Vec<FileRow>) -> Result<Vec<File>, DomainError> {
        files
            .into_iter()
            .map(FileRowMapper::to_file_domain)
            .collect()
    }

    pub fn to_sidecar_domain(sidecar: SidecarRow) -> Result<Sidecar, DomainError> {
        let entry = SidecarDbEntry {
            file_id: sidecar.file_id,
            sidecar_type: sidecar.sidecar_type,
            created_at: sidecar.created_at,
            data: SidecarData {
                path: sidecar.path,
                size_bytes: sidecar.size_bytes as u64,
                mimetype: sidecar.mimetype,
                duration: sidecar.duration_ms.map(|t| Duration::from_millis(t as u64)),
            },
        };
        Ok(Sidecar::from_db(entry))
    }

    pub fn to_sidecar_domain_many(sidecars: Vec<SidecarRow>) -> Result<Vec<Sidecar>, DomainError> {
        sidecars
            .into_iter()
            .map(FileRowMapper::to_sidecar_domain)
            .collect()
    }

    pub fn to_resource(file: File, sidecars: Vec<Sidecar>) -> FileResource {
        FileResource { file, sidecars }
    }

    pub fn to_resource_many(files: Vec<File>, sidecars: Vec<Sidecar>) -> Vec<FileResource> {
        let mut idx: HashMap<FileId, Vec<Sidecar>> = HashMap::with_capacity(files.len());
        for s in sidecars {
            match idx.get_mut(s.file_id()) {
                Some(v) => v.push(s),
                None => {
                    idx.insert(s.file_id().clone(), vec![s]);
                }
            }
        }

        let mut resources = Vec::with_capacity(files.len());
        for f in files {
            let sidecars = idx.get(f.id()).cloned().unwrap_or(Vec::new());
            resources.push(FileResource { file: f, sidecars });
        }

        resources
    }
}

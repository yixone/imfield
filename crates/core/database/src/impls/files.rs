use imfield_models::{
    entities::{File, Sidecar, file::FileResource, sidecar::SidecarType},
    types::id::FileId,
};
use sqlx::SqlitePool;

use crate::{
    DatabaseError,
    ports::files::FilesDatabase,
    types::commands::{DeleteFn, UpdateFn},
};

pub struct SqliteFilesDatabase {
    pool: SqlitePool,
}

impl SqliteFilesDatabase {
    /// Creates a new [`SqliteFilesDatabase`]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl FilesDatabase for SqliteFilesDatabase {
    async fn insert_file(&self, file: &File) -> Result<(), DatabaseError> {
        todo!()
    }

    async fn get(&self, id: &FileId) -> Result<File, DatabaseError> {
        todo!()
    }

    async fn get_sidecar(&self, id: &FileId, ty: SidecarType) -> Result<Sidecar, DatabaseError> {
        todo!()
    }

    async fn get_resource(&self, id: &FileId) -> Result<FileResource, DatabaseError> {
        todo!()
    }

    async fn update_file_fn_boxed(
        &self,
        id: &FileId,
        f: Box<UpdateFn<File>>,
    ) -> Result<File, DatabaseError> {
        todo!()
    }

    async fn update_sidecar_fn_boxed(
        &self,
        id: &FileId,
        ty: SidecarType,
        f: Box<UpdateFn<Sidecar>>,
    ) -> Result<Sidecar, DatabaseError> {
        todo!()
    }

    async fn delete_file_fn_boxed(
        &self,
        id: &FileId,
        f: Box<DeleteFn<File>>,
    ) -> Result<(), DatabaseError> {
        todo!()
    }

    async fn delete_sidecar_fn_boxed(
        &self,
        id: &FileId,
        ty: SidecarType,
        f: Box<DeleteFn<Sidecar>>,
    ) -> Result<(), DatabaseError> {
        todo!()
    }
}

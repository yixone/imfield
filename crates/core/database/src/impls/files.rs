use imfield_models::{
    entities::{File, FileContainer, file::FileContainerResource},
    types::id::{FileContainerId, FileKey},
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

    async fn get_container_by_id(
        &self,
        id: &FileContainerId,
    ) -> Result<FileContainerResource, DatabaseError> {
        todo!()
    }

    async fn get_file(&self, key: &FileKey) -> Result<Option<File>, DatabaseError> {
        todo!()
    }

    async fn update_file_fn_boxed(
        &self,
        key: &FileKey,
        f: Box<UpdateFn<File>>,
    ) -> Result<File, DatabaseError> {
        todo!()
    }

    async fn delete_file_fn_boxed(
        &self,
        key: &FileKey,
        f: Box<DeleteFn<File>>,
    ) -> Result<(), DatabaseError> {
        todo!()
    }

    async fn delete_container_fn_boxed(
        &self,
        id: &FileContainerId,
        f: Box<DeleteFn<FileContainer>>,
    ) -> Result<(), DatabaseError> {
        todo!()
    }
}

use imfield_models::{
    DomainError,
    entities::{File, FileContainer, file::FileContainerResource},
    types::id::{FileContainerId, FileKey},
};

use crate::{
    DatabaseError,
    types::commands::{DeleteFn, UpdateFn},
};

#[async_trait::async_trait]
pub trait FilesDatabase: Send + Sync {
    async fn insert_file(&self, file: &File) -> Result<(), DatabaseError>;

    async fn get_container_by_id(
        &self,
        id: &FileContainerId,
    ) -> Result<FileContainerResource, DatabaseError>;

    async fn get_file(&self, key: &FileKey) -> Result<Option<File>, DatabaseError>;

    async fn update_file_fn_boxed(
        &self,
        key: &FileKey,
        f: Box<UpdateFn<File>>,
    ) -> Result<File, DatabaseError>;

    async fn delete_file_fn_boxed(
        &self,
        key: &FileKey,
        f: Box<DeleteFn<File>>,
    ) -> Result<(), DatabaseError>;

    async fn delete_file(&self, key: &FileKey) -> Result<(), DatabaseError> {
        self.delete_file_fn_boxed(key, Box::new(|_| Ok(()))).await
    }

    async fn delete_container_fn_boxed(
        &self,
        id: &FileContainerId,
        f: Box<DeleteFn<FileContainer>>,
    ) -> Result<(), DatabaseError>;

    async fn delete_container(&self, id: &FileContainerId) -> Result<(), DatabaseError> {
        self.delete_container_fn_boxed(id, Box::new(|_| Ok(())))
            .await
    }
}

#[async_trait::async_trait]
pub trait FilesDatabaseExt: FilesDatabase {
    async fn update_file_fn<Fn>(&self, key: &FileKey, f: Fn) -> Result<File, DatabaseError>
    where
        Fn: FnOnce(&mut File) -> Result<(), DomainError> + Send + 'static,
    {
        self.update_file_fn_boxed(key, Box::new(f)).await
    }

    async fn delete_file_fn<Fn>(&self, key: &FileKey, f: Fn) -> Result<(), DatabaseError>
    where
        Fn: FnOnce(&File) -> Result<(), DomainError> + Send + 'static,
    {
        self.delete_file_fn_boxed(key, Box::new(f)).await
    }

    async fn delete_container_fn<Fn>(
        &self,
        id: &FileContainerId,
        f: Fn,
    ) -> Result<(), DatabaseError>
    where
        Fn: FnOnce(&FileContainer) -> Result<(), DomainError> + Send + 'static,
    {
        self.delete_container_fn_boxed(id, Box::new(f)).await
    }
}

impl<T> FilesDatabaseExt for T where T: FilesDatabase {}

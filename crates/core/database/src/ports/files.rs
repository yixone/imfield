use imfield_models::{
    DomainError,
    entities::{File, Sidecar, file::FileResource, sidecar::SidecarType},
    types::id::FileId,
};

use crate::{
    DatabaseError,
    types::commands::{DeleteFn, UpdateFn},
};

#[async_trait::async_trait]
pub trait FilesDatabase: Send + Sync {
    async fn insert_file(&self, file: &File) -> Result<(), DatabaseError>;

    async fn get(&self, id: &FileId) -> Result<File, DatabaseError>;

    async fn get_sidecar(&self, id: &FileId, ty: SidecarType) -> Result<Sidecar, DatabaseError>;

    async fn get_resource(&self, id: &FileId) -> Result<FileResource, DatabaseError>;

    async fn update_file_fn_boxed(
        &self,
        id: &FileId,
        f: Box<UpdateFn<File>>,
    ) -> Result<File, DatabaseError>;

    async fn update_sidecar_fn_boxed(
        &self,
        id: &FileId,
        ty: SidecarType,
        f: Box<UpdateFn<Sidecar>>,
    ) -> Result<Sidecar, DatabaseError>;

    async fn delete_file_fn_boxed(
        &self,
        id: &FileId,
        f: Box<DeleteFn<File>>,
    ) -> Result<(), DatabaseError>;

    async fn delete_sidecar_fn_boxed(
        &self,
        id: &FileId,
        ty: SidecarType,
        f: Box<DeleteFn<Sidecar>>,
    ) -> Result<(), DatabaseError>;

    async fn delete_file(&self, id: &FileId) -> Result<(), DatabaseError> {
        self.delete_file_fn_boxed(id, Box::new(|_| Ok(()))).await
    }

    async fn delete_sidecar(&self, id: &FileId, ty: SidecarType) -> Result<(), DatabaseError> {
        self.delete_sidecar_fn_boxed(id, ty, Box::new(|_| Ok(())))
            .await
    }
}

#[async_trait::async_trait]
pub trait FilesDatabaseExt: FilesDatabase {
    async fn update_file_fn<Fn>(&self, id: &FileId, f: Fn) -> Result<File, DatabaseError>
    where
        Fn: FnOnce(&mut File) -> Result<(), DomainError> + Send + 'static,
    {
        self.update_file_fn_boxed(id, Box::new(f)).await
    }

    async fn update_sidecar_fn<Fn>(
        &self,
        id: &FileId,
        ty: SidecarType,
        f: Fn,
    ) -> Result<Sidecar, DatabaseError>
    where
        Fn: FnOnce(&mut Sidecar) -> Result<(), DomainError> + Send + 'static,
    {
        self.update_sidecar_fn_boxed(id, ty, Box::new(f)).await
    }

    async fn delete_file_fn<Fn>(&self, id: &FileId, f: Fn) -> Result<(), DatabaseError>
    where
        Fn: FnOnce(&File) -> Result<(), DomainError> + Send + 'static,
    {
        self.delete_file_fn_boxed(id, Box::new(f)).await
    }

    async fn delete_sidecar_fn<Fn>(
        &self,
        id: &FileId,
        ty: SidecarType,
        f: Fn,
    ) -> Result<(), DatabaseError>
    where
        Fn: FnOnce(&Sidecar) -> Result<(), DomainError> + Send + 'static,
    {
        self.delete_sidecar_fn_boxed(id, ty, Box::new(f)).await
    }
}

impl<T> FilesDatabaseExt for T where T: FilesDatabase {}

use imfield_models::{
    DomainError,
    entities::{
        Asset,
        asset::{AssetResource, AssetSortBy, AssetState},
        file::FileResource,
    },
    types::{files::Sha256Checksum, id::AssetId},
};

use crate::{
    DatabaseError,
    types::{
        commands::{DeleteFn, UpdateFn},
        query::{CursorPagination, DeletedVisibility, SortOrder},
    },
};

#[async_trait::async_trait]
pub trait AssetsDatabase: Send + Sync {
    async fn insert(&self, data: (&Asset, &FileResource)) -> Result<(), DatabaseError>;

    async fn get(&self, id: AssetId) -> Result<Option<AssetResource>, DatabaseError> {
        self.get_by_ids(&[id]).await.map(|a| a.into_iter().next())
    }

    async fn get_by_ids(&self, ids: &[AssetId]) -> Result<Vec<AssetResource>, DatabaseError>;

    async fn get_by_state(
        &self,
        cursor: CursorPagination<AssetId>,
        state: AssetState,
    ) -> Result<Vec<AssetId>, DatabaseError>;

    async fn get_by_checksum(&self, c: &Sha256Checksum) -> Result<Vec<Asset>, DatabaseError>;

    async fn get_by_parent_id(
        &self,
        id: AssetId,
        cursor: CursorPagination<AssetId>,
    ) -> Result<Vec<AssetResource>, DatabaseError>;

    async fn get_duplicates_of(
        &self,
        id: AssetId,
        cursor: CursorPagination<AssetId>,
    ) -> Result<Vec<AssetResource>, DatabaseError>;

    async fn list(
        &self,
        cursor: CursorPagination<AssetId>,
        ordering: (AssetSortBy, SortOrder),
        deleted: DeletedVisibility,
    ) -> Result<Vec<AssetResource>, DatabaseError>;

    async fn random(
        &self,
        limit: u32,
        deleted: DeletedVisibility,
    ) -> Result<Vec<AssetResource>, DatabaseError>;

    async fn count(&self, deleted: DeletedVisibility) -> Result<u64, DatabaseError>;

    async fn update_fn_boxed(
        &self,
        id: AssetId,
        f: Box<UpdateFn<Asset>>,
    ) -> Result<Asset, DatabaseError>;

    async fn delete(&self, id: AssetId) -> Result<(), DatabaseError> {
        self.delete_fn_boxed(id, Box::new(|_| Ok(()))).await
    }

    async fn delete_fn_boxed(
        &self,
        id: AssetId,
        f: Box<DeleteFn<Asset>>,
    ) -> Result<(), DatabaseError>;
}

#[async_trait::async_trait]
pub trait AssetsDatabaseExt: AssetsDatabase {
    async fn update_fn<Fn>(&self, id: AssetId, f: Fn) -> Result<Asset, DatabaseError>
    where
        Fn: FnOnce(&mut Asset) -> Result<(), DomainError> + Send + 'static,
    {
        self.update_fn_boxed(id, Box::new(f)).await
    }

    async fn update_state(&self, id: AssetId, state: AssetState) -> Result<Asset, DatabaseError> {
        self.update_fn(id, move |a| {
            a.set_state(state);
            Ok(())
        })
        .await
    }

    async fn delete_fn<Fn>(&self, id: AssetId, f: Fn) -> Result<(), DatabaseError>
    where
        Fn: FnOnce(&Asset) -> Result<(), DomainError> + Send + 'static,
    {
        self.delete_fn_boxed(id, Box::new(f)).await
    }
}

impl<T> AssetsDatabaseExt for T where T: AssetsDatabase {}

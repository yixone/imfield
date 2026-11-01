use imfield_domains::{
    DomainError,
    models::{
        Asset,
        asset::{AssetSortBy, AssetState},
    },
    types::{files::Sha256Checksum, id::AssetId},
};

use crate::{
    DatabaseError,
    types::{
        closure::{DeleteFn, UpdateFn},
        query::{CursorPagination, DeletedVisibility, SortOrder},
        view::{AssetView, FileContainerView},
    },
};

#[async_trait::async_trait]
pub trait AssetsDatabase: Send + Sync {
    async fn insert(&self, data: (&Asset, &FileContainerView)) -> Result<(), DatabaseError>;

    async fn get(&self, id: AssetId) -> Result<Option<AssetView>, DatabaseError> {
        self.get_ids_many(&[id]).await.map(|a| a.into_iter().next())
    }

    async fn get_ids_many(&self, ids: &[AssetId]) -> Result<Vec<AssetView>, DatabaseError>;

    async fn get_by_checksum(&self, c: &Sha256Checksum) -> Result<Vec<Asset>, DatabaseError>;

    async fn list(
        &self,
        cursor: CursorPagination<AssetId>,
        ordering: (AssetSortBy, SortOrder),
        deleted: DeletedVisibility,
    ) -> Result<Vec<AssetView>, DatabaseError>;

    async fn random(
        &self,
        limit: u32,
        deleted: DeletedVisibility,
    ) -> Result<Vec<AssetView>, DatabaseError>;

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
        self.update_fn(id, |a| {
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

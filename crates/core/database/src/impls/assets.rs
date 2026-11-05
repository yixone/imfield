use imfield_models::{
    entities::{
        Asset,
        asset::{AssetSortBy, AssetState},
    },
    types::{files::Sha256Checksum, id::AssetId},
};
use sqlx::SqlitePool;

use crate::{
    DatabaseError,
    ports::assets::AssetsDatabase,
    types::{
        commands::{DeleteFn, UpdateFn},
        query::{CursorPagination, DeletedVisibility, SortOrder},
        view::{AssetView, FileContainerView},
    },
};

pub struct SqliteAssetsDatabase {
    pool: SqlitePool,
}

impl SqliteAssetsDatabase {
    /// Creates a new [`SqliteAssetsDatabase`]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl AssetsDatabase for SqliteAssetsDatabase {
    async fn insert(&self, data: (&Asset, &FileContainerView)) -> Result<(), DatabaseError> {
        todo!()
    }

    async fn get_by_ids(&self, ids: &[AssetId]) -> Result<Vec<AssetView>, DatabaseError> {
        todo!()
    }

    async fn get_by_state(
        &self,
        cursor: CursorPagination<AssetId>,
        state: AssetState,
    ) -> Result<Vec<AssetId>, DatabaseError> {
        todo!()
    }

    async fn get_by_checksum(&self, c: &Sha256Checksum) -> Result<Vec<Asset>, DatabaseError> {
        todo!()
    }

    async fn get_by_parent_id(
        &self,
        id: AssetId,
        cursor: CursorPagination<AssetId>,
    ) -> Result<Vec<AssetView>, DatabaseError> {
        todo!()
    }

    async fn get_duplicates_of(
        &self,
        id: AssetId,
        cursor: CursorPagination<AssetId>,
    ) -> Result<Vec<AssetView>, DatabaseError> {
        todo!()
    }

    async fn list(
        &self,
        cursor: CursorPagination<AssetId>,
        ordering: (AssetSortBy, SortOrder),
        deleted: DeletedVisibility,
    ) -> Result<Vec<AssetView>, DatabaseError> {
        todo!()
    }

    async fn random(
        &self,
        limit: u32,
        deleted: DeletedVisibility,
    ) -> Result<Vec<AssetView>, DatabaseError> {
        todo!()
    }

    async fn count(&self, deleted: DeletedVisibility) -> Result<u64, DatabaseError> {
        todo!()
    }

    async fn update_fn_boxed(
        &self,
        id: AssetId,
        f: Box<UpdateFn<Asset>>,
    ) -> Result<Asset, DatabaseError> {
        todo!()
    }

    async fn delete_fn_boxed(
        &self,
        id: AssetId,
        f: Box<DeleteFn<Asset>>,
    ) -> Result<(), DatabaseError> {
        todo!()
    }
}

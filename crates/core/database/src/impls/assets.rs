use imfield_models::{
    entities::{
        Asset,
        asset::{AssetResource, AssetSortBy, AssetState},
        file::FileResource,
    },
    types::{files::Sha256Checksum, id::AssetId},
};
use sqlx::{QueryBuilder, SqlitePool};

use crate::{
    DatabaseError,
    error::ToDatabaseError,
    impls::queries,
    ports::assets::AssetsDatabase,
    schemas::assets::AssetRow,
    types::{
        commands::{DeleteFn, UpdateFn},
        query::{CursorPagination, DeletedVisibility, SortOrder},
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
    async fn insert(&self, data: (&Asset, &FileResource)) -> Result<(), DatabaseError> {
        let mut tx = self.pool.begin().await.to_db_error()?;

        queries::files::insert_file_container(&data.1.container, &mut tx).await?;
        for f in &data.1.files {
            queries::files::insert_file(f, &mut tx).await?;
        }

        let asset = data.0;
        sqlx::query(
            "
            INSERT INTO assets (
                id, file_container_id,
                accent_color, width, height, thumbhash,
                title, caption, source_url,
                state, state_failure, is_offline,
                parent_id, duplicate_of,
                created_at, updated_at, deleted_at
            )
            VALUES (
                ?, ?,
                ?, ?, ?, ?,
                ?, ?, ?,
                ?, ?, ?,
                ?, ?,
                ?, ?, ?
            )
            ",
        )
        .bind(asset.id())
        .bind(asset.file_id())
        .bind(asset.accent_color())
        .bind(asset.width())
        .bind(asset.height())
        .bind(asset.thumbhash().map(|t| t.as_slice()))
        .bind(asset.title())
        .bind(asset.caption())
        .bind(asset.source_url())
        .bind(asset.state().to_string())
        .bind(match asset.state() {
            AssetState::Failed(f) => Some(f),
            _ => None,
        })
        .bind(asset.is_offline())
        .bind(asset.parent_id())
        .bind(asset.duplicate_of())
        .bind(asset.created_at())
        .bind(asset.updated_at())
        .bind(asset.deleted_at())
        .execute(&mut *tx)
        .await
        .to_db_error()?;

        sqlx::query(
            "
            INSERT INTO assets_features (
                id, updated_at,
                p_hash, a_hash
            )
            VALUES (
                ?, ?,
                ?, ?
            )
            ",
        )
        .bind(asset.id())
        .bind(asset.similarity_features().updated_at())
        .bind(asset.similarity_features().p_hash())
        .bind(asset.similarity_features().a_hash())
        .execute(&mut *tx)
        .await
        .to_db_error()?;

        tx.commit().await.to_db_error()?;

        Ok(())
    }

    async fn get_by_ids(&self, ids: &[AssetId]) -> Result<Vec<AssetResource>, DatabaseError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        // Uses a transaction to obtain a consistent collectable domain
        let mut tx = self.pool.begin().await.to_db_error()?;

        let assets: Vec<AssetRow> = {
            let mut qb = QueryBuilder::new(
                "
                SELECT * FROM assets
                WHERE id IN (
                ",
            );

            let mut sep = qb.separated(", ");
            for id in ids {
                sep.push_bind(id);
            }
            sep.push_unseparated(")");

            qb.build_query_as()
                .fetch_all(&mut *tx)
                .await
                .to_db_error()?
        };

        tx.commit().await.to_db_error()?;

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
    ) -> Result<Vec<AssetResource>, DatabaseError> {
        todo!()
    }

    async fn get_duplicates_of(
        &self,
        id: AssetId,
        cursor: CursorPagination<AssetId>,
    ) -> Result<Vec<AssetResource>, DatabaseError> {
        todo!()
    }

    async fn list(
        &self,
        cursor: CursorPagination<AssetId>,
        ordering: (AssetSortBy, SortOrder),
        deleted: DeletedVisibility,
    ) -> Result<Vec<AssetResource>, DatabaseError> {
        todo!()
    }

    async fn random(
        &self,
        limit: u32,
        deleted: DeletedVisibility,
    ) -> Result<Vec<AssetResource>, DatabaseError> {
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

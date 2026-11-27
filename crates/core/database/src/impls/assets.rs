use std::time::Duration;

use chrono::Utc;
use imfield_models::{
    DomainError,
    entities::{
        Asset, File,
        asset::{AssetResource, AssetSortBy, AssetState},
    },
    types::{files::Sha256Checksum, id::AssetId},
};
use sqlx::{QueryBuilder, SqlitePool};

use crate::{
    DatabaseError,
    error::ToDatabaseError,
    impls::{queries, utils::hydrator::AssetHydrator},
    ports::AssetsDatabase,
    schemas::assets::AssetRowMapper,
    types::{
        commands::{DeleteFn, UpdateFn},
        query::{CursorPagination, DeletedVisibility, Pagination, SortOrder},
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
    async fn insert(&self, data: (&Asset, &File)) -> Result<(), DatabaseError> {
        let mut tx = self.pool.begin().await.to_db_error()?;

        queries::files::insert_file(data.1, &mut tx).await?;

        let asset = data.0;
        sqlx::query(
            "
            INSERT INTO assets (
                id, file_id,
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

        let mut conn = self.pool.acquire().await.to_db_error()?;

        let assets = {
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
                .fetch_all(&mut *conn)
                .await
                .to_db_error()?
        };

        AssetHydrator::hydrate_asset_resource_many(assets, &mut conn).await
    }

    async fn list(
        &self,
        pagination: CursorPagination<AssetId>,
        ordering: (AssetSortBy, SortOrder),
        deleted: DeletedVisibility,
    ) -> Result<Vec<AssetResource>, DatabaseError> {
        if pagination.limit() == 0 {
            return Ok(vec![]);
        }

        let mut query = {
            let mut qb = QueryBuilder::new(
                "
                SELECT a.* FROM assets AS a
                LEFT JOIN files AS f
                    ON f.id = a.file_id
                WHERE 1 = 1
                ",
            );

            // Applies the visibility of deleted items
            match deleted {
                DeletedVisibility::Show => (),
                DeletedVisibility::Hide => {
                    qb.push(" AND a.deleted_at IS null ");
                }
                DeletedVisibility::DeletedOnly => {
                    qb.push(" AND a.deleted_at IS NOT null ");
                }
            }

            let (order_by, ordering) = ordering;
            let order_by_col = match order_by {
                AssetSortBy::CreatedAt => "a.created_at",
                AssetSortBy::DeletedAt => "a.deleted_at",
                // NOTE: COALESCE(f.size_bytes, 0) could have been used here,
                // but according to the scheme, the asset should always have a file
                AssetSortBy::OriginalSize => "f.size_bytes",
            };

            // Applies cursor-based pagination
            if let Some(cursor) = pagination.cursor() {
                // `AND (<col>, id)`
                qb.push(" AND ( ");
                qb.push(order_by_col);
                qb.push(", a.id )");

                // `>` / `<`
                qb.push(match ordering {
                    SortOrder::Asc => " > ",
                    SortOrder::Desc => " < ",
                });

                // Executes a subquery to retrieve cursor data
                qb.push("( SELECT ");
                qb.push(order_by_col);
                qb.push(", a.id FROM assets AS a ");
                qb.push("LEFT JOIN files AS f ON f.id = a.file_id");
                qb.push(" WHERE a.id = ");
                qb.push_bind(cursor);
                qb.push(")");
            }

            qb.push(" ORDER BY ");

            qb.push(order_by_col);
            qb.push(" ");
            qb.push(ordering.as_str());

            qb.push(", a.id ");
            qb.push(ordering.as_str());

            qb.push(" LIMIT ");
            qb.push_bind(pagination.limit() as i64);

            qb
        };

        let mut conn = self.pool.acquire().await.to_db_error()?;

        let assets = query
            .build_query_as()
            .fetch_all(&mut *conn)
            .await
            .to_db_error()?;

        AssetHydrator::hydrate_asset_resource_many(assets, &mut conn).await
    }

    async fn get_by_state(
        &self,
        pagination: Pagination,
        state: AssetState,
    ) -> Result<Vec<AssetId>, DatabaseError> {
        if pagination.limit() == 0 {
            return Ok(vec![]);
        }

        let mut conn = self.pool.acquire().await.to_db_error()?;

        let ids = sqlx::query_scalar(
            "
            SELECT id FROM assets
            WHERE state = ?
            ORDER BY created_at ASC, id ASC
            LIMIT ? OFFSET ?
            ",
        )
        .bind(state.as_str())
        .bind(pagination.limit() as i64)
        .bind(pagination.try_offset()? as i64)
        .fetch_all(&mut *conn)
        .await
        .to_db_error()?;

        Ok(ids)
    }

    async fn get_for_processing(
        &self,
        pagination: Pagination,
    ) -> Result<Vec<AssetId>, DatabaseError> {
        if pagination.limit() == 0 {
            return Ok(vec![]);
        }

        let mut conn = self.pool.acquire().await.to_db_error()?;

        let ids = sqlx::query_scalar(
            "
            SELECT a.id FROM assets AS a
            INNER JOIN assets_features AS af
                ON af.id = a.id
            WHERE 
                a.deleted_at IS null
                AND (
                    -- The asset has not yet been processed
                    a.state = 'pending'

                    -- The asset was not processed due to an error or hang
                    OR (
                        a.state IN ('processing', 'failed')
                        AND (unixepoch('now') - unixepoch(a.updated_at)) >= ?
                    )
                )
            ORDER BY a.created_at ASC
            LIMIT ? OFFSET ?
            ",
        )
        .bind(Duration::from_mins(5).as_secs() as i64)
        .bind(pagination.limit() as i64)
        .bind(pagination.try_offset()? as i64)
        .fetch_all(&mut *conn)
        .await
        .to_db_error()?;

        Ok(ids)
    }

    async fn get_deleted_older(
        &self,
        older: chrono::Duration,
        pagination: Pagination,
    ) -> Result<Vec<AssetId>, DatabaseError> {
        if pagination.limit() == 0 {
            return Ok(vec![]);
        }

        let mut conn = self.pool.acquire().await.to_db_error()?;

        let ids = sqlx::query_scalar(
            "
            SELECT a.id FROM assets AS a
            WHERE 
                a.deleted_at IS NOT null
                AND unixepoch('now') - unixepoch(a.updated_at)) >= ?
            ORDER BY a.deleted_at ASC
            LIMIT ? OFFSET ?
            ",
        )
        .bind(older.num_seconds())
        .bind(pagination.limit() as i64)
        .bind(pagination.try_offset()? as i64)
        .fetch_all(&mut *conn)
        .await
        .to_db_error()?;

        Ok(ids)
    }

    async fn get_by_checksum(&self, c: &Sha256Checksum) -> Result<Option<Asset>, DatabaseError> {
        let mut conn = self.pool.acquire().await.to_db_error()?;

        let row = sqlx::query_as(
            "
            SELECT a.* FROM assets AS a
            LEFT JOIN files AS f
                ON f.id = a.file_id
            WHERE f.checksum = ?
            ORDER BY f.size_bytes DESC, a.created_at ASC
            ",
        )
        .bind(c.into_inner().to_vec())
        .fetch_optional(&mut *conn)
        .await
        .to_db_error()?;

        let Some(row) = row else {
            return Ok(None);
        };

        AssetHydrator::hydrate_asset(row, &mut conn).await.map(Some)
    }

    async fn get_by_parent_id(
        &self,
        id: AssetId,
        pagination: CursorPagination<AssetId>,
    ) -> Result<Vec<AssetResource>, DatabaseError> {
        let mut query = {
            let mut qb = QueryBuilder::new(
                "
                SELECT * FROM assets
                WHERE parent_id = 
                ",
            );
            qb.push_bind(id);

            // Applies cursor-based pagination
            if let Some(cursor) = pagination.cursor() {
                qb.push(
                    "
                    AND (created_at, id) < (
                        SELECT created_at, id FROM assets
                        WHERE id = 
                    ",
                );
                qb.push_bind(cursor);
                qb.push(")");
            }

            qb.push(" ORDER BY created_at DESC, id DESC ");

            qb.push(" LIMIT ");
            qb.push_bind(pagination.limit() as i64);

            qb
        };

        let mut conn = self.pool.acquire().await.to_db_error()?;

        let assets = query
            .build_query_as()
            .fetch_all(&mut *conn)
            .await
            .to_db_error()?;

        AssetHydrator::hydrate_asset_resource_many(assets, &mut conn).await
    }

    async fn get_duplicates_of(
        &self,
        id: AssetId,
        pagination: CursorPagination<AssetId>,
    ) -> Result<Vec<AssetResource>, DatabaseError> {
        let mut query = {
            let mut qb = QueryBuilder::new(
                "
                SELECT * FROM assets
                WHERE duplicate_of = 
                ",
            );
            qb.push_bind(id);

            // Applies cursor-based pagination
            if let Some(cursor) = pagination.cursor() {
                qb.push(
                    "
                    AND (created_at, id) < (
                        SELECT created_at, id FROM assets
                        WHERE id = 
                    ",
                );
                qb.push_bind(cursor);
                qb.push(")");
            }

            qb.push(" ORDER BY created_at DESC, id DESC ");

            qb.push(" LIMIT ");
            qb.push_bind(pagination.limit() as i64);

            qb
        };

        let mut conn = self.pool.acquire().await.to_db_error()?;

        let assets = query
            .build_query_as()
            .fetch_all(&mut *conn)
            .await
            .to_db_error()?;

        AssetHydrator::hydrate_asset_resource_many(assets, &mut conn).await
    }

    async fn random(
        &self,
        limit: u32,
        deleted: DeletedVisibility,
    ) -> Result<Vec<AssetResource>, DatabaseError> {
        let mut query = {
            let mut qb = QueryBuilder::new(
                "
                SELECT * FROM assets
                WHERE 1 = 1
                ",
            );

            // Applies the visibility of deleted items
            match deleted {
                DeletedVisibility::Show => (),
                DeletedVisibility::Hide => {
                    qb.push(" AND deleted_at IS null ");
                }
                DeletedVisibility::DeletedOnly => {
                    qb.push(" AND deleted_at IS NOT null ");
                }
            }

            qb.push(" ORDER BY RANDOM() ");

            qb.push(" LIMIT ");
            qb.push_bind(limit);

            qb
        };

        let mut conn = self.pool.acquire().await.to_db_error()?;

        let assets = query
            .build_query_as()
            .fetch_all(&mut *conn)
            .await
            .to_db_error()?;

        AssetHydrator::hydrate_asset_resource_many(assets, &mut conn).await
    }

    async fn count(&self, deleted: DeletedVisibility) -> Result<u64, DatabaseError> {
        let mut query = {
            let mut qb = QueryBuilder::new(
                "
                SELECT COUNT(id) FROM assets
                WHERE 1 = 1
                ",
            );

            // Applies the visibility of deleted items
            match deleted {
                DeletedVisibility::Show => (),
                DeletedVisibility::Hide => {
                    qb.push(" AND deleted_at IS null ");
                }
                DeletedVisibility::DeletedOnly => {
                    qb.push(" AND deleted_at IS NOT null ");
                }
            }

            qb
        };

        let mut conn = self.pool.acquire().await.to_db_error()?;

        let count = query
            .build_query_scalar::<i64>()
            .fetch_one(&mut *conn)
            .await
            .to_db_error()?;

        Ok(count as u64)
    }

    async fn update_fn_boxed(
        &self,
        id: AssetId,
        f: Box<UpdateFn<Asset>>,
    ) -> Result<Asset, DatabaseError> {
        let mut tx = self.pool.begin().await.to_db_error()?;

        let row = sqlx::query_as(
            "
            SELECT * FROM assets
            WHERE id = ?
            ",
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .to_db_error()?;
        let mut asset = AssetHydrator::hydrate_asset(row, &mut tx).await?;

        f(&mut asset)?;

        let updated = sqlx::query_as(
            "
            UPDATE assets
            SET 
                accent_color = ?, width = ?, height = ?, thumbhash = ?,
                title = ?, caption = ?, source_url = ?,
                state = ?, state_failure = ?, is_offline = ?,
                parent_id = ?, duplicate_of = ?,
                updated_at = ?, deleted_at = ?
            WHERE id = ?  
            RETURNING *
            ",
        )
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
        .bind(asset.updated_at())
        .bind(asset.deleted_at())
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .to_db_error()?;

        let updated_features = sqlx::query_as(
            "
            UPDATE assets_features
            SET 
                updated_at = ?,
                p_hash = ?, a_hash = ?
            WHERE id = ?
            RETURNING *

        ",
        )
        .bind(asset.similarity_features().updated_at())
        .bind(asset.similarity_features().p_hash())
        .bind(asset.similarity_features().a_hash())
        .bind(asset.id())
        .fetch_one(&mut *tx)
        .await
        .to_db_error()?;

        let ret = AssetRowMapper::to_domain(updated, updated_features)?;

        tx.commit().await.to_db_error()?;

        Ok(ret)
    }

    async fn delete_fn_boxed(
        &self,
        id: AssetId,
        f: Box<DeleteFn<Asset>>,
    ) -> Result<(), DatabaseError> {
        let mut tx = self.pool.begin().await.to_db_error()?;

        let row = sqlx::query_as(
            "
            SELECT * FROM assets
            WHERE id = ?
            ",
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .to_db_error()?;
        let asset = AssetHydrator::hydrate_asset(row, &mut tx).await?;

        f(&asset)?;

        let result = sqlx::query(
            "
            DELETE FROM assets
            WHERE id = ?
            ",
        )
        .bind(asset.id())
        .execute(&mut *tx)
        .await
        .to_db_error()?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound.into());
        }

        tx.commit().await.to_db_error()?;

        Ok(())
    }

    async fn try_start_processing(
        &self,
        id: AssetId,
    ) -> Result<Option<AssetResource>, DatabaseError> {
        let mut tx = self.pool.begin().await.to_db_error()?;

        let updated = sqlx::query(
            "
            UPDATE assets
            SET 
                state = ?, updated_at = ?
            WHERE id = ? AND state != ?
            ",
        )
        .bind(AssetState::Processing.as_str())
        .bind(Some(Utc::now()))
        .bind(id)
        .bind(AssetState::Processing.as_str())
        .execute(&mut *tx)
        .await
        .to_db_error()?;

        if updated.rows_affected() == 0 {
            tx.rollback().await.to_db_error()?;
            return Ok(None);
        }

        let asset = sqlx::query_as(
            "
            SELECT * FROM assets
            WHERE id = ?
            ",
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .to_db_error()?;

        let a = AssetHydrator::hydrate_asset_resource_many(vec![asset], &mut tx)
            .await?
            .into_iter()
            .next();

        tx.commit().await.to_db_error()?;

        Ok(a)
    }
}

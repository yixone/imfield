use imfield_models::{
    DomainError,
    entities::{File, Sidecar, file::FileResource, sidecar::SidecarType},
    types::id::FileId,
};
use sqlx::SqlitePool;

use crate::{
    DatabaseError,
    error::ToDatabaseError,
    impls::{queries, utils::hydrator::FileHydrator},
    ports::files::FilesDatabase,
    schemas::files::FileRowMapper,
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
    async fn insert_sidecar(&self, sidecar: &Sidecar) -> Result<(), DatabaseError> {
        let mut tx = self.pool.begin().await.to_db_error()?;
        queries::files::insert_sidecar(sidecar, &mut tx).await?;
        tx.commit().await.to_db_error()
    }

    async fn get_file(&self, id: &FileId) -> Result<File, DatabaseError> {
        let mut conn = self.pool.acquire().await.to_db_error()?;
        let file = queries::files::load_file(id, &mut conn)
            .await?
            .map(FileRowMapper::to_file_domain)
            .ok_or(DomainError::NotFound)??;
        Ok(file)
    }

    async fn get_sidecar(&self, id: &FileId, ty: SidecarType) -> Result<Sidecar, DatabaseError> {
        let mut conn = self.pool.acquire().await.to_db_error()?;
        let sidecar = queries::files::load_sidecar(id, ty, &mut conn)
            .await?
            .map(FileRowMapper::to_sidecar_domain)
            .ok_or(DomainError::NotFound)??;
        Ok(sidecar)
    }

    async fn get_resource(&self, id: &FileId) -> Result<FileResource, DatabaseError> {
        let mut conn = self.pool.acquire().await.to_db_error()?;
        let file = queries::files::load_file(id, &mut conn)
            .await?
            .ok_or(DomainError::NotFound)?;

        FileHydrator::hydrate_file_resource(file, &mut conn).await
    }

    async fn update_file_fn_boxed(
        &self,
        id: &FileId,
        f: Box<UpdateFn<File>>,
    ) -> Result<File, DatabaseError> {
        let mut tx = self.pool.begin().await.to_db_error()?;

        let mut file = queries::files::load_file(id, &mut tx)
            .await?
            .map(FileRowMapper::to_file_domain)
            .ok_or(DomainError::NotFound)??;

        f(&mut file)?;

        let updated = sqlx::query_as(
            "
            UPDATE files
            SET path = ?, duration_ms = ?
            WHERE id = ?
            RETURNING *
            ",
        )
        .bind(file.path())
        .bind(file.duration().map(|d| d.as_millis() as i64))
        .bind(file.id())
        .fetch_one(&mut *tx)
        .await
        .to_db_error()?;

        tx.commit().await.to_db_error()?;

        Ok(FileRowMapper::to_file_domain(updated)?)
    }

    async fn update_sidecar_fn_boxed(
        &self,
        id: &FileId,
        ty: SidecarType,
        f: Box<UpdateFn<Sidecar>>,
    ) -> Result<Sidecar, DatabaseError> {
        let mut tx = self.pool.begin().await.to_db_error()?;

        let mut sidecar = queries::files::load_sidecar(id, ty, &mut tx)
            .await?
            .map(FileRowMapper::to_sidecar_domain)
            .ok_or(DomainError::NotFound)??;

        f(&mut sidecar)?;

        let updated = sqlx::query_as(
            "
            UPDATE sidecars
            SET path = ?, duration_ms = ?
            WHERE file_id = ? AND sidecar_type = ?
            RETURNING *
            ",
        )
        .bind(sidecar.path())
        .bind(sidecar.duration().map(|d| d.as_millis() as i64))
        .bind(sidecar.file_id())
        .bind(sidecar.sidecar_type())
        .fetch_one(&mut *tx)
        .await
        .to_db_error()?;

        tx.commit().await.to_db_error()?;

        Ok(FileRowMapper::to_sidecar_domain(updated)?)
    }

    async fn delete_file_fn_boxed(
        &self,
        id: &FileId,
        f: Box<DeleteFn<File>>,
    ) -> Result<(), DatabaseError> {
        let mut tx = self.pool.begin().await.to_db_error()?;

        let file = queries::files::load_file(id, &mut tx)
            .await?
            .map(FileRowMapper::to_file_domain)
            .ok_or(DomainError::NotFound)??;

        f(&file)?;

        let result = sqlx::query(
            "
            DELETE FROM files
            WHERE id = ?
            ",
        )
        .bind(file.id())
        .execute(&mut *tx)
        .await
        .to_db_error()?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound.into());
        }

        tx.commit().await.to_db_error()?;

        Ok(())
    }

    async fn delete_sidecar_fn_boxed(
        &self,
        id: &FileId,
        ty: SidecarType,
        f: Box<DeleteFn<Sidecar>>,
    ) -> Result<(), DatabaseError> {
        let mut tx = self.pool.begin().await.to_db_error()?;

        let sidecar = queries::files::load_sidecar(id, ty, &mut tx)
            .await?
            .map(FileRowMapper::to_sidecar_domain)
            .ok_or(DomainError::NotFound)??;

        f(&sidecar)?;

        let result = sqlx::query(
            "
            DELETE FROM sidecars
            WHERE file_id = ? AND sidecar_type = ?
            ",
        )
        .bind(sidecar.file_id())
        .bind(sidecar.sidecar_type())
        .execute(&mut *tx)
        .await
        .to_db_error()?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound.into());
        }

        tx.commit().await.to_db_error()?;

        Ok(())
    }
}

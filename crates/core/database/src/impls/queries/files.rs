use imfield_models::{
    entities::{File, Sidecar},
    types::id::FileId,
};
use sqlx::{QueryBuilder, SqliteConnection};

use crate::{
    DatabaseError,
    error::ToDatabaseError,
    schemas::files::{FileRow, SidecarRow},
};

pub async fn insert_file(file: &File, conn: &mut SqliteConnection) -> Result<(), DatabaseError> {
    sqlx::query(
        "
        INSERT INTO files (
            id, created_at,
            path, size_bytes,
            mimetype, mimekind,
            duration_ms,
            original_filename, checksum
        )
        VALUES (
            ?, ?,
            ?, ?,
            ?, ?,
            ?,
            ?, ?
        )
        ",
    )
    .bind(file.id())
    .bind(file.created_at())
    .bind(file.path())
    .bind(file.size_bytes() as i64)
    .bind(file.mimetype())
    .bind(file.mimekind())
    .bind(file.duration().map(|d| d.as_millis() as i64))
    .bind(file.original_filename())
    .bind(file.checksum().into_inner().to_vec())
    .execute(conn)
    .await
    .to_db_error()?;

    Ok(())
}

pub async fn insert_sidecar(
    sidecar: &Sidecar,
    conn: &mut SqliteConnection,
) -> Result<(), DatabaseError> {
    sqlx::query(
        "
        INSERT INTO sidecars (
            file_id, sidecar_type,
            created_at,
            path, size_bytes,
            mimetype,
            duration_ms
        )
        VALUES (
            ?, ?,
            ?,
            ?, ?,
            ?,
            ?
        )
        ",
    )
    .bind(sidecar.id())
    .bind(sidecar.sidecar_type())
    .bind(sidecar.created_at())
    .bind(sidecar.path())
    .bind(sidecar.size_bytes() as i64)
    .bind(sidecar.mimetype())
    .bind(sidecar.duration().map(|d| d.as_millis() as i64))
    .execute(conn)
    .await
    .to_db_error()?;

    Ok(())
}

pub async fn load_files_many(
    ids: &[FileId],
    conn: &mut SqliteConnection,
) -> Result<Vec<FileRow>, DatabaseError> {
    if ids.is_empty() {
        return Ok(vec![]);
    }

    let mut qb = QueryBuilder::new(
        "
        SELECT * FROM files
        WHERE id IN (
        ",
    );

    let mut sep = qb.separated(", ");
    for id in ids {
        sep.push_bind(id);
    }
    sep.push_unseparated(")");

    qb.build_query_as().fetch_all(conn).await.to_db_error()
}

pub async fn load_file(
    id: &FileId,
    conn: &mut SqliteConnection,
) -> Result<Option<FileRow>, DatabaseError> {
    load_files_many(std::slice::from_ref(id), conn)
        .await
        .map(|f| f.into_iter().next())
}

pub async fn load_files_sidecars_many(
    ids: &[FileId],
    conn: &mut SqliteConnection,
) -> Result<Vec<SidecarRow>, DatabaseError> {
    if ids.is_empty() {
        return Ok(vec![]);
    }

    let mut qb = QueryBuilder::new(
        "
        SELECT * FROM sidecars
        WHERE file_id IN (
        ",
    );

    let mut sep = qb.separated(", ");
    for id in ids {
        sep.push_bind(id);
    }
    sep.push_unseparated(")");

    qb.build_query_as().fetch_all(conn).await.to_db_error()
}

pub async fn load_file_sidecars(
    id: &FileId,
    conn: &mut SqliteConnection,
) -> Result<Vec<SidecarRow>, DatabaseError> {
    load_files_sidecars_many(std::slice::from_ref(id), conn).await
}

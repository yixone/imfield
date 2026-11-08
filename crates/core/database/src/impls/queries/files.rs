use imfield_models::{
    entities::{File, FileContainer},
    types::id::FileContainerId,
};
use sqlx::{QueryBuilder, SqliteConnection, query::Query};

use crate::{
    DatabaseError,
    error::ToDatabaseError,
    schemas::files::{FileContainerRow, FileRow},
};

pub async fn insert_file_container(
    cont: &FileContainer,
    conn: &mut SqliteConnection,
) -> Result<(), DatabaseError> {
    sqlx::query(
        "
        INSERT INTO file_containers (
            id,
            original_checksum, original_filename, original_mimekind
        )
        VALUES (
            ?,
            ?, ?, ?
        )
        ",
    )
    .bind(cont.id())
    .bind(cont.original_checksum().into_inner().to_vec())
    .bind(cont.original_filename())
    .bind(cont.original_mimekind())
    .execute(conn)
    .await
    .to_db_error()?;

    Ok(())
}

pub async fn insert_file(file: &File, conn: &mut SqliteConnection) -> Result<(), DatabaseError> {
    sqlx::query(
        "
        INSERT INTO files (
            container_id, variant,
            created_at,
            path,
            size_bytes, mimetype,
            duration_ms
        )
        VALUES (
            ?, ?,
            ?,
            ?,
            ?, ?,
            ?
        )
        ",
    )
    .bind(file.key().container())
    .bind(file.key().variant())
    .bind(file.created_at())
    .bind(file.path())
    .bind(file.size_bytes() as i64)
    .bind(file.mimetype())
    .bind(file.duration().map(|f| f.as_millis() as i64))
    .execute(conn)
    .await
    .to_db_error()?;

    Ok(())
}

pub async fn load_container_files(
    id: &FileContainerId,
    conn: &mut SqliteConnection,
) -> Result<Vec<FileRow>, DatabaseError> {
    load_container_files_many(std::slice::from_ref(id), conn).await
}

pub async fn load_container_files_many(
    ids: &[FileContainerId],
    conn: &mut SqliteConnection,
) -> Result<Vec<FileRow>, DatabaseError> {
    if ids.is_empty() {
        return Ok(vec![]);
    }

    let mut qb = QueryBuilder::new(
        "
        SELECT * FROM files
        WHERE container_id IN (
        ",
    );

    let mut sep = qb.separated(", ");
    for id in ids {
        sep.push_bind(id);
    }
    sep.push_unseparated(")");

    qb.build_query_as().fetch_all(conn).await.to_db_error()
}

pub async fn load_container(
    id: &FileContainerId,
    conn: &mut SqliteConnection,
) -> Result<Option<FileContainerRow>, DatabaseError> {
    load_container_many(std::slice::from_ref(id), conn)
        .await
        .map(|c| c.into_iter().next())
}

pub async fn load_container_many(
    ids: &[FileContainerId],
    conn: &mut SqliteConnection,
) -> Result<Vec<FileContainerRow>, DatabaseError> {
    if ids.is_empty() {
        return Ok(vec![]);
    }

    let mut qb = QueryBuilder::new(
        "
        SELECT * FROM file_containers
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

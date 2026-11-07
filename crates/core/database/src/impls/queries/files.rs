use imfield_models::entities::{File, FileContainer};
use sqlx::SqliteConnection;

use crate::{DatabaseError, error::ToDatabaseError};

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

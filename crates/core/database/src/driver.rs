use std::path::Path;

use sqlx::{
    SqlitePool,
    sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};

use crate::{DatabaseError, error::ToDatabaseError};

/// SQLite database migrator
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

/// An opened SQLite database
///
/// Provides access to the database through a connection pool
pub struct SqliteDatabase {
    pool: SqlitePool,
}

impl SqliteDatabase {
    /// Opens an [`SqliteDatabase`] from a file at the specified path
    ///
    /// Creates the database file and its parent directories if they do not exist
    pub async fn open(p: impl AsRef<Path>) -> Result<Self, DatabaseError> {
        let path = p.as_ref();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).to_db_error()?;
        }

        let opts = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .auto_vacuum(SqliteAutoVacuum::Incremental)
            .journal_mode(SqliteJournalMode::Wal);

        let pool = SqlitePoolOptions::new()
            .connect_with(opts)
            .await
            .to_db_error()?;

        Ok(SqliteDatabase { pool })
    }

    /// Applies migrations to an open database
    pub async fn migrate(&self) -> Result<(), DatabaseError> {
        MIGRATOR.run(&self.pool).await.to_db_error()
    }
}

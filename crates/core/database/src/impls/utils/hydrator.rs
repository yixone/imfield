use imfield_models::{
    DomainError,
    entities::{Asset, asset::AssetResource, file::FileResource},
};
use sqlx::SqliteConnection;

use crate::{
    DatabaseError,
    impls::queries,
    schemas::{
        assets::{AssetRow, AssetRowMapper},
        files::{FileRow, FileRowMapper},
    },
};

pub struct AssetHydrator;

impl AssetHydrator {
    pub async fn hydrate_assets_many(
        rows: Vec<AssetRow>,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<Asset>, DatabaseError> {
        let ids = rows.iter().map(|a| a.id).collect::<Vec<_>>();

        let features = queries::assets::load_asset_features_many(&ids, conn).await?;
        let domains = AssetRowMapper::to_domain_many(rows, features)?;

        Ok(domains)
    }

    pub async fn hydrate_asset(
        row: AssetRow,
        conn: &mut SqliteConnection,
    ) -> Result<Asset, DatabaseError> {
        let features = queries::assets::load_asset_features(row.id, conn)
            .await?
            .ok_or(DomainError::PersistenceError)?;
        let domain = AssetRowMapper::to_domain(row, features)?;
        Ok(domain)
    }

    pub async fn hydrate_asset_resource_many(
        rows: Vec<AssetRow>,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<AssetResource>, DatabaseError> {
        let assets = AssetHydrator::hydrate_assets_many(rows, conn).await?;

        let ids = assets
            .iter()
            .map(|a| a.file_id().clone())
            .collect::<Vec<_>>();
        let files = queries::files::load_files_many(&ids, conn).await?;

        let files_res = FileHydrator::hydrate_file_resource_many(files, conn).await?;

        let res = AssetRowMapper::to_resource_many(assets, files_res)?;
        Ok(res)
    }
}

pub struct FileHydrator;

impl FileHydrator {
    pub async fn hydrate_file_resource_many(
        rows: Vec<FileRow>,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<FileResource>, DatabaseError> {
        let ids = rows.iter().map(|f| f.id.clone()).collect::<Vec<_>>();
        let sidecars_rows = queries::files::load_files_sidecars_many(&ids, conn).await?;

        let files = FileRowMapper::to_file_domain_many(rows)?;
        let sidecars = FileRowMapper::to_sidecar_domain_many(sidecars_rows)?;

        let res = FileRowMapper::to_resource_many(files, sidecars);
        Ok(res)
    }

    pub async fn hydrate_file_resource(
        row: FileRow,
        conn: &mut SqliteConnection,
    ) -> Result<FileResource, DatabaseError> {
        let sidecars_rows = queries::files::load_file_sidecars(&row.id, conn).await?;

        let file = FileRowMapper::to_file_domain(row)?;
        let sidecars = FileRowMapper::to_sidecar_domain_many(sidecars_rows)?;

        let res = FileRowMapper::to_resource(file, sidecars);
        Ok(res)
    }
}

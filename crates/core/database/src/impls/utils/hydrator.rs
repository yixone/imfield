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
        files::{FileContainerRow, FileRowMapper},
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

    pub async fn hydrate_asset_resource(
        row: AssetRow,
        conn: &mut SqliteConnection,
    ) -> Result<AssetResource, DatabaseError> {
        let asset = AssetHydrator::hydrate_asset(row, conn).await?;

        let file_cont = queries::files::load_container(asset.file_id(), conn)
            .await?
            .ok_or(DomainError::PersistenceError)?;
        let files = FileHydrator::hydrate_container_resource(file_cont, conn).await?;

        let res = AssetRowMapper::to_resource(asset, files);
        Ok(res)
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
        let file_cont = queries::files::load_container_many(&ids, conn).await?;
        let files = FileHydrator::hydrate_container_resource_many(file_cont, conn).await?;

        let res = AssetRowMapper::to_resource_many(assets, files)?;
        Ok(res)
    }
}

pub struct FileHydrator;

impl FileHydrator {
    pub async fn hydrate_container_resource(
        row: FileContainerRow,
        conn: &mut SqliteConnection,
    ) -> Result<FileResource, DatabaseError> {
        let files = queries::files::load_container_files(&row.id, conn).await?;

        let container = FileRowMapper::to_container_domain(row)?;
        let files = FileRowMapper::to_file_domain_many(files)?;

        let res = FileRowMapper::to_resource(container, files);
        Ok(res)
    }

    pub async fn hydrate_container_resource_many(
        rows: Vec<FileContainerRow>,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<FileResource>, DatabaseError> {
        let ids = rows.iter().map(|g| g.id.clone()).collect::<Vec<_>>();
        let files = queries::files::load_container_files_many(&ids, conn).await?;

        let containers = FileRowMapper::to_container_domain_many(rows)?;
        let files = FileRowMapper::to_file_domain_many(files)?;

        let res = FileRowMapper::to_resource_many(containers, files);
        Ok(res)
    }
}

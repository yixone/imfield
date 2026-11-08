use imfield_models::types::id::AssetId;
use sqlx::{QueryBuilder, SqliteConnection};

use crate::{DatabaseError, error::ToDatabaseError, schemas::assets::AssetSimilarityFeaturesRow};

pub async fn load_asset_features(
    id: AssetId,
    conn: &mut SqliteConnection,
) -> Result<Option<AssetSimilarityFeaturesRow>, DatabaseError> {
    load_asset_features_many(&[id], conn)
        .await
        .map(|f| f.into_iter().next())
}

pub async fn load_asset_features_many(
    ids: &[AssetId],
    conn: &mut SqliteConnection,
) -> Result<Vec<AssetSimilarityFeaturesRow>, DatabaseError> {
    let mut qb = QueryBuilder::new(
        "
        SELECT * FROM assets_features
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

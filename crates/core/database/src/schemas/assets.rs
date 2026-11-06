use chrono::{DateTime, Utc};
use imfield_models::{
    entities::asset::AssetStateFailure,
    types::{
        id::{AssetId, FileContainerId},
        media::{Color, PerceptualHash},
    },
};
use sqlx::prelude::FromRow;

pub type AssetFullRow = (AssetRow, AssetSimilarityFeaturesRow);

#[derive(FromRow, Clone)]
pub struct AssetRow {
    pub id: AssetId,
    pub file_container_id: FileContainerId,
    pub accent_color: Option<Color>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub thumbhash: Option<Vec<u8>>,
    pub title: Option<String>,
    pub caption: Option<String>,
    pub source_url: Option<String>,
    pub state: String,
    pub state_failure: Option<AssetStateFailure>,
    pub is_offline: bool,
    pub parent_id: Option<AssetId>,
    pub duplicate_of: Option<AssetId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(FromRow, Clone)]
pub struct AssetSimilarityFeaturesRow {
    pub id: AssetId,
    pub updated_at: Option<DateTime<Utc>>,
    pub p_hash: Option<PerceptualHash>,
    pub a_hash: Option<PerceptualHash>,
}

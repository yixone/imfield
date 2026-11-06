use chrono::{DateTime, Utc};
use imfield_models::{
    DomainError,
    entities::{
        Asset,
        asset::{AssetData, AssetPersistence, AssetState, AssetStateFailure},
    },
    types::{
        id::{AssetId, FileContainerId},
        media::{Color, PerceptualHash, ThumbHash},
    },
};
use sqlx::prelude::FromRow;

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

pub struct AssetRows {
    pub asset: AssetRow,
    pub similarity_features: AssetSimilarityFeaturesRow,
}

impl From<Asset> for AssetRows {
    fn from(a: Asset) -> Self {
        let p = AssetPersistence::from(a);
        AssetRows {
            asset: AssetRow {
                id: p.id,
                file_container_id: p.file_container,
                accent_color: p.accent_color,
                width: p.dimension.map(|(w, _)| w as i32),
                height: p.dimension.map(|(_, h)| h as i32),
                thumbhash: p.thumbhash.map(|t| t.into_inner()),
                title: p.data.title,
                caption: p.data.caption,
                source_url: p.data.source_url,
                state: p.state.to_string(),
                state_failure: match p.state {
                    AssetState::Failed(f) => Some(f),
                    _ => None,
                },
                is_offline: p.is_offline,
                parent_id: p.data.parent_id,
                duplicate_of: p.data.duplicate_of,
                created_at: p.created_at,
                updated_at: p.updated_at,
                deleted_at: p.deleted_at,
            },
            similarity_features: AssetSimilarityFeaturesRow {
                id: p.id,
                updated_at: p.features_updated_at,
                p_hash: p.p_hash,
                a_hash: p.a_hash,
            },
        }
    }
}

impl TryFrom<AssetRows> for Asset {
    type Error = DomainError;

    fn try_from(rows: AssetRows) -> Result<Self, Self::Error> {
        let AssetRows {
            asset: a,
            similarity_features: sf,
        } = rows;

        let state = AssetState::from_persistence(&a.state, a.state_failure)?;

        let persistence = AssetPersistence {
            id: a.id,
            file_container: a.file_container_id,
            data: AssetData {
                title: a.title,
                caption: a.caption,
                source_url: a.source_url,
                parent_id: a.parent_id,
                duplicate_of: a.duplicate_of,
            },
            accent_color: a.accent_color,
            dimension: match (a.width, a.height) {
                (Some(w), Some(h)) => Some((w as u32, h as u32)),
                (Some(_), None) | (None, Some(_)) => return Err(DomainError::PersistenceError),
                _ => None,
            },
            thumbhash: a.thumbhash.map(ThumbHash::new),
            state,
            is_offline: a.is_offline,
            created_at: a.created_at,
            updated_at: a.updated_at,
            deleted_at: a.deleted_at,
            features_updated_at: sf.updated_at,
            p_hash: sf.p_hash,
            a_hash: sf.a_hash,
        };

        Asset::try_from(persistence)
    }
}

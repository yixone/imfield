use std::collections::HashMap;

use chrono::{DateTime, Utc};
use imfield_models::{
    DomainError,
    entities::{
        Asset,
        asset::{AssetData, AssetResource, AssetState, AssetStateFailure},
        file::FileResource,
    },
    types::{
        id::{AssetId, FileContainerId},
        media::{Color, PerceptualHash, ThumbHash},
        store::AssetStoreEntry,
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

/// Maps asset database rows to domain entities and resource models
pub struct AssetRowMapper;

impl AssetRowMapper {
    /// Convert asset database rows into a domain entity
    pub fn to_domain(
        asset: AssetRow,
        features: AssetSimilarityFeaturesRow,
    ) -> Result<Asset, DomainError> {
        let entry = AssetStoreEntry {
            id: asset.id,
            file_container: asset.file_container_id,
            data: AssetData {
                title: asset.title,
                caption: asset.caption,
                source_url: asset.source_url,
                parent_id: asset.parent_id,
                duplicate_of: asset.duplicate_of,
            },
            accent_color: asset.accent_color,
            dimension: match (asset.width, asset.height) {
                (Some(w), Some(h)) => Some((w as u32, h as u32)),
                (Some(_), None) | (None, Some(_)) => return Err(DomainError::PersistenceError),
                (None, None) => None,
            },
            thumbhash: asset.thumbhash.map(ThumbHash::new),
            state: AssetState::from_persistence(&asset.state, asset.state_failure)?,
            is_offline: asset.is_offline,
            created_at: asset.created_at,
            updated_at: asset.updated_at,
            deleted_at: asset.deleted_at,
            features_updated_at: features.updated_at,
            p_hash: features.p_hash,
            a_hash: features.a_hash,
        };
        Ok(Asset::from_stored(entry))
    }

    /// Converts multiple asset database rows into domain entities
    pub fn to_domain_many(
        assets: Vec<AssetRow>,
        features: Vec<AssetSimilarityFeaturesRow>,
    ) -> Result<Vec<Asset>, DomainError> {
        let mut index = HashMap::with_capacity(features.len());
        for f in features {
            if index.contains_key(&f.id) {
                continue;
            }

            index.insert(f.id, f);
        }

        let mut domains = Vec::with_capacity(assets.len());
        for a in assets {
            let f = index.get(&a.id).ok_or(DomainError::PersistenceError)?;
            let row = AssetRowMapper::to_domain(a, f.clone())?;

            domains.push(row);
        }

        Ok(domains)
    }

    /// Creates an asset resource from a domain entity and its file container
    pub fn to_resource(asset: Asset, files: FileResource) -> AssetResource {
        AssetResource { asset, files }
    }

    /// Creates multiple asset resources by matching file containers to assets
    pub fn to_resource_many(
        assets: Vec<Asset>,
        files: Vec<FileResource>,
    ) -> Result<Vec<AssetResource>, DomainError> {
        let mut index = HashMap::with_capacity(files.len());
        for f in files {
            if index.contains_key(f.container.id()) {
                continue;
            }
            index.insert(f.container.id().clone(), f);
        }

        let mut resources = Vec::with_capacity(assets.len());
        for a in assets {
            let f = index
                .get(a.file_id())
                .ok_or(DomainError::PersistenceError)?;

            resources.push(AssetResource {
                asset: a,
                files: f.clone(),
            });
        }

        Ok(resources)
    }

    /// Converts a domain entity into it corresponding database rows
    pub fn from_domain(asset: Asset) -> (AssetRow, AssetSimilarityFeaturesRow) {
        let entry = asset.to_stored();
        (
            AssetRow {
                id: entry.id,
                file_container_id: entry.file_container,
                accent_color: entry.accent_color,
                width: entry.dimension.map(|(w, _)| w as i32),
                height: entry.dimension.map(|(_, h)| h as i32),
                thumbhash: entry.thumbhash.map(|t| t.into_inner()),
                title: entry.data.title,
                caption: entry.data.caption,
                source_url: entry.data.source_url,
                state: entry.state.to_string(),
                state_failure: match entry.state {
                    AssetState::Failed(f) => Some(f),
                    _ => None,
                },
                is_offline: entry.is_offline,
                parent_id: entry.data.parent_id,
                duplicate_of: entry.data.duplicate_of,
                created_at: entry.created_at,
                updated_at: entry.updated_at,
                deleted_at: entry.deleted_at,
            },
            AssetSimilarityFeaturesRow {
                id: entry.id,
                updated_at: entry.features_updated_at,
                p_hash: entry.p_hash,
                a_hash: entry.a_hash,
            },
        )
    }
}

use chrono::{DateTime, Utc};

use crate::types::id::AssetId;

#[derive(Debug)]
pub struct AssetEntity {
    pub(crate) id: AssetId,

    pub(crate) title: Option<String>,
    pub(crate) caption: Option<String>,

    pub(crate) state: AssetState,
    pub(crate) is_offline: bool,

    pub(crate) parent_id: Option<AssetId>,
    pub(crate) duplicate_of: Option<AssetId>,

    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: Option<DateTime<Utc>>,
    pub(crate) deleted_at: Option<DateTime<Utc>>,
}

impl AssetEntity {
    pub fn id(&self) -> AssetId {
        self.id
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn caption(&self) -> Option<&str> {
        self.caption.as_deref()
    }

    pub fn state(&self) -> &AssetState {
        &self.state
    }

    pub fn is_offline(&self) -> bool {
        self.is_offline
    }

    pub fn parent_id(&self) -> Option<AssetId> {
        self.parent_id
    }

    pub fn duplicate_of(&self) -> Option<AssetId> {
        self.duplicate_of
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.updated_at
    }

    pub fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }
}

#[derive(Debug)]
pub enum AssetState {
    Pending,
    Processing,
    Ready,
    Failed(AssetStateFailure),
}

#[derive(Debug)]
pub enum AssetStateFailure {
    Unknown,
}

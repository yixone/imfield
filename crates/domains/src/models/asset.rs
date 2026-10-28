use chrono::{DateTime, Utc};
use imfield_mimetype::MimeKind;

use crate::{
    DomainError,
    types::{
        files::Sha256Checksum,
        id::{AssetId, FileGroupId},
        media::{Color, PerceptualHash},
        url::Url,
    },
    utils::constrains,
};

/// Asset entity
///
/// An asset is the core entity in `imfield`,
/// representing content managed by the application
#[derive(Debug)]
pub struct AssetEntity {
    /// Unique snowflake ID for an asset
    pub(crate) id: AssetId,

    /// Set of calculated asset features
    pub(crate) features: AssetFeatures,

    /// Identifier for a group of files associated with an asset
    pub(crate) file_group_id: FileGroupId,
    /// SHA256 Checksum for deduplication and verification of the original file
    pub(crate) checksum: Sha256Checksum,
    /// Original filename of the file uploaded by the user
    pub(crate) original_filename: Option<String>,
    /// Type of the original file uploaded by the user
    pub(crate) original_mimekind: MimeKind,

    /// Asset name
    pub(crate) title: Option<String>,
    /// Asset description
    pub(crate) caption: Option<String>,
    /// The source from which the asset was obtained
    pub(crate) source: Option<AssetSource>,

    /// Asset lifecycle state
    pub(crate) state: AssetState,
    /// If `true`, the original file has been lost from storage and the asset cannot be used
    pub(crate) is_offline: bool,

    /// Parent asset ID
    pub(crate) parent_id: Option<AssetId>,
    /// ID of the asset that this asset is considered a duplicate of
    pub(crate) duplicate_of: Option<AssetId>,

    /// Asset creation time
    pub(crate) created_at: DateTime<Utc>,
    /// Time of the asset's last modification
    pub(crate) updated_at: Option<DateTime<Utc>>,
    /// Optional asset deletion time
    ///
    /// If `Some`, the asset is considered deleted
    pub(crate) deleted_at: Option<DateTime<Utc>>,
}

impl AssetEntity {
    pub fn id(&self) -> AssetId {
        self.id
    }

    pub fn features(&self) -> &AssetFeatures {
        &self.features
    }

    pub fn file_group_id(&self) -> &FileGroupId {
        &self.file_group_id
    }

    pub fn checksum(&self) -> Sha256Checksum {
        self.checksum
    }

    pub fn original_filename(&self) -> Option<&str> {
        self.original_filename.as_deref()
    }

    pub fn original_mimekind(&self) -> MimeKind {
        self.original_mimekind
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn set_title(&mut self, title: Option<String>) -> Result<(), DomainError> {
        if let Some(title) = &title {
            constrains::ensure_str_len(title, 64)?;
        }
        self.title = title;
        Ok(())
    }

    pub fn caption(&self) -> Option<&str> {
        self.caption.as_deref()
    }

    pub fn set_caption(&mut self, caption: Option<String>) -> Result<(), DomainError> {
        if let Some(caption) = &caption {
            constrains::ensure_str_len(caption, 512)?;
        }
        self.caption = caption;
        Ok(())
    }

    pub fn source(&self) -> Option<&AssetSource> {
        self.source.as_ref()
    }

    pub fn set_source(&mut self, source: Option<AssetSource>) {
        self.source = source;
    }

    pub fn state(&self) -> &AssetState {
        &self.state
    }

    pub fn is_offline(&self) -> bool {
        self.is_offline
    }

    pub fn set_is_offline(&mut self, is_offline: bool) {
        self.is_offline = is_offline;
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

    pub fn set_deleted(&mut self) {
        self.deleted_at = Some(Utc::now());
    }

    pub fn unset_deleted(&mut self) {
        self.deleted_at = None;
    }
}

/// Asset lifecycle state
#[derive(Debug)]
pub enum AssetState {
    /// The asset has been created and is awaiting processing
    Pending,
    /// The asset is being processed
    Processing,
    /// The asset has been processed and is ready for use
    Ready,
    /// Processing of the asset failed
    Failed(AssetStateFailure),
}

/// Description of the reason for an asset processing error
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize,),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(rename_all = "snake_case"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetStateFailure {
    /// The asset has invalid or unsupported dimensions
    InvalidDimension,

    /// The asset has an invalid or unsupported duration
    InvalidDuration,

    /// Media metadata could not be read or parsed
    ProbeFailed,

    /// The asset's original file is unavailable in storage
    OriginalFileUnavailable,

    /// Unknown or otherwise unclassified processing error
    ///
    /// See the logs for underlying details
    Unknown,
}

/// The source from which the asset was obtained
#[derive(Debug, Clone, PartialEq)]
pub enum AssetSource {
    /// Asset source URL
    Url { url: Url },
    /// Prompt data used to create the asset via Ai
    Ai {
        model: Option<String>,
        prompt: String,
    },
}

/// Derived features computed from an asset
///
/// Attributes are used for a service that finds similar
/// assets and for detecting duplicates
#[derive(Debug, Default)]
pub struct AssetFeatures {
    /// Asset accent color
    pub(crate) accent_color: Option<Color>,

    /// Asset width and height in pixels
    pub(crate) dimension: Option<(u32, u32)>,

    /// Asset perceptual hash
    pub(crate) p_hash: Option<PerceptualHash>,
    /// Asset average hash
    pub(crate) a_hash: Option<PerceptualHash>,
}

impl AssetFeatures {
    pub fn accent_color(&self) -> Option<Color> {
        self.accent_color
    }

    pub fn dimension(&self) -> Option<(u32, u32)> {
        self.dimension
    }

    pub fn p_hash(&self) -> Option<PerceptualHash> {
        self.p_hash
    }

    pub fn a_hash(&self) -> Option<PerceptualHash> {
        self.a_hash
    }
}

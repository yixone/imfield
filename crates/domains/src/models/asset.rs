use chrono::{DateTime, Utc};
use imfield_mimetype::MimeKind;

use crate::{
    DomainError,
    types::{
        files::Sha256Checksum,
        id::{AssetId, FileGroupId},
        media::{Color, PerceptualHash, ThumbHash},
        url::Url,
    },
};

/// An asset managed by `imfield`
///
/// Represent a single unit of content together with its metadata,
/// derived features, original file information, and state
#[derive(Debug)]
pub struct AssetEntity {
    /// Unique identifier of the asset
    pub(crate) id: AssetId,

    /// Asset accent color
    pub(crate) accent_color: Option<Color>,
    /// Asset width and height in pixels
    pub(crate) dimension: Option<(u32, u32)>,
    /// ThumbHash calculated for the asset
    pub(crate) thumbhash: Option<ThumbHash>,

    /// Identifier of the file group containing this asset's files
    pub(crate) file_group_id: FileGroupId,
    /// SHA-256 checksum of the original file, used for
    /// content verification
    pub(crate) checksum: Sha256Checksum,
    /// Original filename, if available
    pub(crate) original_filename: Option<String>,
    /// MIME kind of the original file
    pub(crate) original_mimekind: MimeKind,

    /// User-defined title of the asset
    pub(crate) title: Option<String>,
    /// User-defined description of the asset
    pub(crate) caption: Option<String>,
    /// URL of the source from which the asset was obtained, if known
    pub(crate) source_url: Option<Url>,

    /// Current lifecycle state of the asset
    pub(crate) state: AssetState,
    /// Whether the original file is missing from storage
    ///
    /// An offline asset may not be available for normal use until its
    /// original file is restored
    pub(crate) is_offline: bool,

    pub(crate) similarity_features: AssetSimilarityFeatures,

    /// Identifier of the parent [`AssetEntity`], if this asset
    /// belongs to another asset
    pub(crate) parent_id: Option<AssetId>,
    /// Identifier of the [`AssetEntity`] this asset
    /// is considered a duplicate of
    pub(crate) duplicate_of: Option<AssetId>,

    /// Timestamp at which the asset was created
    pub(crate) created_at: DateTime<Utc>,
    /// Timestamp of the most recent modification, if any
    pub(crate) updated_at: Option<DateTime<Utc>>,
    /// Timestamp at which the asset was marked as deleted, if applicable
    ///
    /// A value of `Some` indicates that the asset is logically deleted
    pub(crate) deleted_at: Option<DateTime<Utc>>,
}

/// Data required to create a new [`AssetEntity`]
pub struct NewAssetData {
    /// Initial title of the asset
    pub title: Option<String>,
    /// Initial description of the asset
    pub caption: Option<String>,
    /// URL of the source from which the asset was obtained, if known
    pub source_url: Option<Url>,

    /// SHA-256 checksum of the original file
    pub checksum: Sha256Checksum,

    /// Identifier of the parent asset, if any
    pub parent_id: Option<AssetId>,
    /// Identifier of the asset this asset is considered a duplicate of, if any
    pub duplicate_of: Option<AssetId>,
}

impl AssetEntity {
    /// Creates a new [`AssetEntity`] and validates its initial data
    ///
    /// The asset is initialized in the [`AssetState::Pending`] state,
    /// with default features and the current timestamp
    pub fn new(id: AssetId, data: NewAssetData) -> Result<Self, DomainError> {
        // Validates asset data
        validate_asset_title(&data.title)?;
        validate_asset_caption(&data.caption)?;

        // Verifies that an asset does not reference itself
        if data.parent_id == Some(id) || data.duplicate_of == Some(id) {
            return Err(DomainError::RelationViolation);
        }

        let features = AssetSimilarityFeatures::default();
        let entity = AssetEntity {
            id,
            similarity_features: features,
            accent_color: None,
            dimension: None,
            thumbhash: None,
            file_group_id: todo!(),
            checksum: data.checksum,
            original_filename: todo!(),
            original_mimekind: todo!(),
            title: data.title,
            caption: data.caption,
            source_url: data.source_url,
            state: AssetState::Pending,
            is_offline: false,
            parent_id: data.parent_id,
            duplicate_of: data.duplicate_of,
            created_at: Utc::now(),
            updated_at: None,
            deleted_at: None,
        };

        Ok(entity)
    }

    /// Returns the asset's identifier
    pub fn id(&self) -> AssetId {
        self.id
    }

    /// Sets the asset's dimension in pixels
    pub fn set_dimension(&mut self, dim: Option<(u32, u32)>) {
        // TODO: Add validation for (e.g. for (0,0) or (99999999, 999999999))
        self.dimension = dim;
        self.refresh_updated_at();
    }

    /// Returns the asset's width in pixels, if available
    pub fn width(&self) -> Option<u32> {
        self.dimension.map(|(w, _)| w)
    }

    /// Returns the asset's height in pixels, if available
    pub fn height(&self) -> Option<u32> {
        self.dimension.map(|(_, h)| h)
    }

    /// Returns the asset's aspect ratio (width / height)
    ///
    /// Returns `None` if the asset dimensions are unavailable
    pub fn aspect_ratio(&self) -> Option<f32> {
        if let Some((w, h)) = self.dimension {
            // Prevents division by `0`
            if h == 0 {
                return None;
            }
            Some(w as f32 / h as f32)
        } else {
            None
        }
    }

    /// Returns the asset's accent color, if available
    pub fn accent_color(&self) -> Option<Color> {
        self.accent_color
    }

    /// Sets the asset's accent color
    pub fn set_accent_color(&mut self, color: Option<Color>) {
        self.accent_color = color;
        self.refresh_updated_at();
    }

    /// Returns the asset's ThumbHash, if available
    pub fn thumbhash(&self) -> Option<&ThumbHash> {
        self.thumbhash.as_ref()
    }

    /// Sets the asset's ThumbHash
    pub fn set_thumbhash(&mut self, th: Option<ThumbHash>) {
        self.thumbhash = th;
        self.refresh_updated_at();
    }

    /// Returns the asset's features used to similarity search
    pub fn similarity_features(&self) -> &AssetSimilarityFeatures {
        &self.similarity_features
    }

    /// Returns a mutable reference to the asset's features used to similarity search
    pub fn similarity_features_mut(&mut self) -> &mut AssetSimilarityFeatures {
        &mut self.similarity_features
    }

    /// Returns the identifier of the asset's file group
    pub fn file_group_id(&self) -> &FileGroupId {
        &self.file_group_id
    }

    /// Returns the SHA-256 checksum of the original file
    pub fn checksum(&self) -> Sha256Checksum {
        self.checksum
    }

    /// Returns the orignal filename, if available
    pub fn original_filename(&self) -> Option<&str> {
        self.original_filename.as_deref()
    }

    /// Returns the MIME kind of the original file
    pub fn original_mimekind(&self) -> MimeKind {
        self.original_mimekind
    }

    /// Returns the asset's title, if set
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Validates and updates the asset's title
    pub fn set_title(&mut self, title: Option<String>) -> Result<(), DomainError> {
        validate_asset_title(&title)?;
        self.title = title;
        self.refresh_updated_at();
        Ok(())
    }

    /// Returns the asset's description, if set
    pub fn caption(&self) -> Option<&str> {
        self.caption.as_deref()
    }

    /// Validates and updates the asset's description
    pub fn set_caption(&mut self, caption: Option<String>) -> Result<(), DomainError> {
        validate_asset_caption(&caption)?;
        self.caption = caption;
        self.refresh_updated_at();
        Ok(())
    }

    /// Returns the URL of the asset's source, if known
    pub fn source_url(&self) -> Option<&Url> {
        self.source_url.as_ref()
    }

    /// Sets the URL of the asset's source
    ///
    /// Pass `None` to clear the source URL
    pub fn set_source_url(&mut self, source: Option<Url>) {
        self.source_url = source;
        self.refresh_updated_at();
    }

    /// Returns the asset's current lifecycle state
    pub fn state(&self) -> &AssetState {
        &self.state
    }

    /// Returns `true` if the original file is missing from storage
    pub fn is_offline(&self) -> bool {
        self.is_offline
    }

    /// Updates whether the original file is missing from storage
    pub fn set_is_offline(&mut self, is_offline: bool) {
        self.is_offline = is_offline;
        self.refresh_updated_at();
    }

    /// Returns the identifier of the parent asset, if any
    pub fn parent_id(&self) -> Option<AssetId> {
        self.parent_id
    }

    /// Sets the parent asset
    ///
    /// Pass `None` to remove the parent association
    ///
    /// Returns [`DomainError::RelationViolation`] if the specified
    /// parent is the asset itself
    pub fn set_parent_id(&mut self, parent: Option<AssetId>) -> Result<(), DomainError> {
        if parent == Some(self.id) {
            return Err(DomainError::RelationViolation);
        }
        self.parent_id = parent;
        self.refresh_updated_at();
        Ok(())
    }

    /// Returns the identifier of the asset this asset duplicates, if any
    pub fn duplicate_of(&self) -> Option<AssetId> {
        self.duplicate_of
    }

    /// Sets the asset this asset is considered a duplicate of
    ///
    /// Pass `None` to clear the duplicate association
    ///
    /// Returns [`DomainError::RelationViolation`] if the specified
    /// asset is the asset itself
    pub fn set_duplicate_of(&mut self, duplicate_of: Option<AssetId>) -> Result<(), DomainError> {
        if duplicate_of == Some(self.id) {
            return Err(DomainError::RelationViolation);
        }
        self.duplicate_of = duplicate_of;
        self.refresh_updated_at();
        Ok(())
    }

    /// Returns the asset's creation timestamp in UTC
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Returns the timestamp of the asset's most recent modification, if any
    pub fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.updated_at
    }

    /// Returns the timestamp at which the asset was marked as deleted, if any
    pub fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }

    /// Marks the asset as deleted by recording the current UTC timestamp
    ///
    /// Returns [`DomainError::EntityDeleted`] if the asset is already marked as deleted
    pub fn set_deleted(&mut self) -> Result<(), DomainError> {
        self.ensure_not_deleted()?;
        self.deleted_at = Some(Utc::now());
        self.refresh_updated_at();
        Ok(())
    }

    /// Returns `Ok(())` if the asset is not deleted, or [`DomainError::EntityDeleted`] otherwise
    pub fn ensure_not_deleted(&self) -> Result<(), DomainError> {
        if self.is_deleted() {
            return Err(DomainError::EntityDeleted);
        }
        Ok(())
    }

    /// Restores the asset by clearing its deletion timestamp
    pub fn unset_deleted(&mut self) {
        self.deleted_at = None;
        self.refresh_updated_at();
    }

    /// Returns `true` if this asset is marked as deleted
    #[inline]
    pub fn is_deleted(&self) -> bool {
        self.deleted_at().is_some()
    }

    /// Sets the current timestamp as the asset's update time
    fn refresh_updated_at(&mut self) {
        self.updated_at = Some(Utc::now());
    }

    /// Updates the asset's lifecycle state
    pub fn set_state(&mut self, state: AssetState) {
        self.state = state;
        self.refresh_updated_at();
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

// TODO: Add `features_updated_at`
#[derive(Debug, Default)]
pub struct AssetSimilarityFeatures {
    /// Asset perceptual hash
    pub(crate) p_hash: Option<PerceptualHash>,
    /// Asset average hash
    pub(crate) a_hash: Option<PerceptualHash>,
}

impl AssetSimilarityFeatures {
    pub fn p_hash(&self) -> Option<PerceptualHash> {
        self.p_hash
    }

    pub fn a_hash(&self) -> Option<PerceptualHash> {
        self.a_hash
    }
}

/// Validates the asset name
fn validate_asset_title(title: &Option<String>) -> Result<(), DomainError> {
    if let Some(title) = title
        && title.len() > 64
    {
        return Err(DomainError::BadInput);
    }
    Ok(())
}

/// Validates the asset caption
fn validate_asset_caption(caption: &Option<String>) -> Result<(), DomainError> {
    if let Some(caption) = caption
        && caption.len() > 1024
    {
        return Err(DomainError::BadInput);
    }
    Ok(())
}

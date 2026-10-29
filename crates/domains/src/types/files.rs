/// 32-byte SHA256 checksum for bitwise file comparison
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sha256Checksum(pub(crate) [u8; 32]);

impl Sha256Checksum {
    /// Creates a new [`Sha256Checksum`]
    pub fn new(b: [u8; 32]) -> Self {
        Self(b)
    }

    /// Returns the internal value of the checksum
    pub fn into_inner(&self) -> [u8; 32] {
        self.0
    }

    /// Returns the [`hex`] representation of the [`Sha256Checksum`]
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

/// Media file variant
///
/// May represent the original media file or a generated
/// derivative optimized for a specific use case
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum FileVariant {
    /// Original media file uploaded by the user
    Original,

    /// Low-resolution preview for quick asset display
    Thumbnail,
}

impl FileVariant {
    /// Returns the [`FileVariant`] as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Original => "original",
            Self::Thumbnail => "thumbnail",
        }
    }
}

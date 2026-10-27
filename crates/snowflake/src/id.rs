use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};

/// 64-bit unique identifier
///
/// SnowflakeID consists of a `41-bit` timestamp, an `8-bit` node ID, and a `14-bit` sequence number.
/// The sign bit of the integer is always zero, so the value of id is always greater than zero
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SnowflakeId(pub(crate) i64);

impl<T> From<T> for SnowflakeId
where
    T: Into<i64>,
{
    fn from(int: T) -> Self {
        SnowflakeId(int.into())
    }
}

impl std::fmt::Display for SnowflakeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// [`SnowflakeId`] converted to a string via [`base64`] encoding
///
/// For more information, see the [`SnowflakeId`] documentation
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SnowflakeIdStr(pub(crate) String);

impl From<SnowflakeId> for SnowflakeIdStr {
    fn from(id: SnowflakeId) -> Self {
        let encoded = BASE64_URL_SAFE_NO_PAD.encode(id.0.to_le_bytes());
        SnowflakeIdStr(encoded)
    }
}

impl AsRef<str> for SnowflakeIdStr {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SnowflakeIdStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

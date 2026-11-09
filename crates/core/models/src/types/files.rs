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

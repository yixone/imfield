/// A color stored as a 32-bit integer (8 bits for each color channel)
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
pub struct Color(pub(crate) i32);

impl Color {
    /// Creates a new [`Color`] from RGB channels
    pub fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Color(((red as i32) << 16) | ((green as i32) << 8) | (blue as i32))
    }

    /// Returns the color as a RGB channels tuple
    pub const fn rgb(self) -> (u8, u8, u8) {
        (
            ((self.0 >> 16) & 0xFF) as u8,
            ((self.0 >> 8) & 0xFF) as u8,
            ((self.0) & 0xFF) as u8,
        )
    }

    /// Returns the color as a hex string in the format `#ffffff`
    pub fn hex(self) -> String {
        let (r, g, b) = self.rgb();
        format!("#{r:02x}{g:02x}{b:02x}")
    }
}

/// A perceptual hash that is used for visual comparison of media
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
pub struct PerceptualHash(pub(crate) i64);

impl PerceptualHash {
    /// Creates a new [`PerceptualHash`]
    pub fn new(hash: i64) -> Self {
        PerceptualHash(hash)
    }

    /// Returns the internal value of the hash
    pub fn into_inner(&self) -> i64 {
        self.0
    }

    /// Finds the Hamming distance between two perceptual hashes
    pub fn hamming(&self, rhs: Self) -> u32 {
        (self.0 ^ rhs.0).count_ones()
    }
}

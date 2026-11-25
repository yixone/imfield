use std::io::Cursor;

use image::{DynamicImage, GenericImageView, ImageReader, Pixel};
use imfield_models::types::media::{Color, ImageHash, ThumbHash};
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::image::{
    ImageError,
    hash::{a_hash, p_hash, thumbhash},
    types::ResizeFilter,
};

/// Maximum allowed width or height of a decoded image, in pixels
const MAX_IMAGE_DIMENSION: u32 = 2048 * 6;
/// Maximum allowed numbers of pixels in a decoded image
const MAX_IMAGE_PIXELS: u64 = (MAX_IMAGE_DIMENSION as u64 * MAX_IMAGE_DIMENSION as u64) / 2;

/// An opened and decoded image
pub struct Image {
    dim: (u32, u32),
    inner: DynamicImage,
}

impl Image {
    /// Reads and decodes an image from an asyncronous reader
    ///
    /// The image format is guessed from the input data. Its dimensions are
    /// checked before decoding to reject images exceeding the configured
    /// dimension or pixel count limits
    pub async fn from_async_read<R>(mut reader: R) -> Result<Self, ImageError>
    where
        R: AsyncRead + Unpin,
    {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;

        // Performs a dimension check before decoding
        let cursor = Cursor::new(&mut buf);
        let reader = ImageReader::new(cursor).with_guessed_format()?;

        let (w, h) = reader.into_dimensions()?;
        validate_dimension(w, h)?;

        // After the checks, decodes the image
        let reader = ImageReader::new(Cursor::new(buf));
        let decoded = reader.with_guessed_format()?.decode()?;

        Ok(Image {
            dim: (decoded.width(), decoded.height()),
            inner: decoded,
        })
    }

    /// Returns the image width in pixels
    pub fn width(&self) -> u32 {
        self.dim.0
    }

    /// Returns the image height in pixels
    pub fn height(&self) -> u32 {
        self.dim.1
    }

    /// Resizes the image to the specified width while preserving its aspect ratio
    pub fn resize_by_width(&self, nw: u32, filter: ResizeFilter) -> Result<Image, ImageError> {
        let (w, h) = self.dim;
        let nh = (h as u64 * nw as u64) / w as u64;
        validate_dimension(nw, nh as u32)?;

        Ok(Image {
            dim: (nw, nh as u32),
            inner: self.inner.resize_exact(nw, nh as u32, filter.into()),
        })
    }

    /// Resizes the image to the specified height while preserving its aspect ratio
    pub fn resize_by_height(&self, nh: u32, filter: ResizeFilter) -> Result<Image, ImageError> {
        let (w, h) = self.dim;
        let nw = (w as u64 * nh as u64) / h as u64;
        validate_dimension(nw as u32, nh)?;

        Ok(Image {
            dim: (nw as u32, nh),
            inner: self.inner.resize_exact(nw as u32, nh, filter.into()),
        })
    }

    /// Resizes the image to the specified dimensions
    pub fn resize(&self, nw: u32, nh: u32, filter: ResizeFilter) -> Result<Image, ImageError> {
        validate_dimension(nw, nh)?;

        Ok(Image {
            dim: (nw, nh),
            inner: self.inner.resize_exact(nw, nh, filter.into()),
        })
    }

    /// Returns `true` if the image is wider than it is tall
    pub fn is_landscape(&self) -> bool {
        self.width() > self.height()
    }

    /// Generates a ThumbHash representation of the image
    pub fn thumbhash(&self) -> ThumbHash {
        let rgba = self.inner.pixels().map(|p| p.2).collect::<Vec<_>>();
        thumbhash(&rgba, self.width() as usize, self.height() as usize)
    }

    /// Calculates the average hash (aHash) of the image
    pub fn a_hash(&self) -> Result<ImageHash, ImageError> {
        let pixels = self.prepare_luma_matrix()?;
        Ok(a_hash(pixels))
    }

    /// Calculates the perceptual hash (pHash) of the image
    pub fn p_hash(&self) -> Result<ImageHash, ImageError> {
        let pixels = self.prepare_luma_matrix()?;
        Ok(p_hash(pixels))
    }

    /// Extracts an average color representing the image
    pub fn average_color(&self) -> Color {
        // TODO: Implement a proper algorithm for detecting accent
        let resized = self.inner.resize_exact(1, 1, ResizeFilter::Triangle.into());
        let [r, g, b] = resized.get_pixel(0, 0).to_rgb().0;
        Color::from_rgb(r, g, b)
    }

    fn prepare_luma_matrix<T, K>(&self) -> Result<T, ImageError>
    where
        K: TryFrom<Vec<u8>>,
        T: TryFrom<Vec<K>>,
        T::Error: std::fmt::Debug,
        K::Error: std::fmt::Debug,
    {
        let luma = self.inner.to_luma8();
        luma.rows()
            .map(|i| {
                i.map(|p| p.0[0])
                    .collect::<Vec<_>>()
                    .try_into()
                    .map_err(|_| ImageError::InvalidDimension)
            })
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|_| ImageError::InvalidDimension)
    }
}

fn validate_dimension(w: u32, h: u32) -> Result<(), ImageError> {
    if w == 0 || h == 0 {
        return Err(ImageError::InvalidDimension);
    }

    if w > MAX_IMAGE_DIMENSION
        || h > MAX_IMAGE_DIMENSION
        || (w as u64 * h as u64) > MAX_IMAGE_PIXELS
    {
        return Err(ImageError::DimensionLimitExceeded);
    }

    Ok(())
}

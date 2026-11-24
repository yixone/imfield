use std::io::Cursor;

use image::{DynamicImage, GenericImageView, imageops::FilterType};
use imfield_models::types::media::ThumbHash;
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::image::{ImageError, hash::thumbhash};

/// Opened and decoded image
pub struct Image {
    dim: (u32, u32),
    inner: DynamicImage,
}

impl Image {
    pub async fn from_async_read<R>(mut reader: R) -> Result<Self, ImageError>
    where
        R: AsyncRead + Unpin,
    {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;

        let im_reader = image::ImageReader::new(Cursor::new(buf));
        let decoded = im_reader.with_guessed_format()?.decode()?;

        Ok(Image {
            dim: (decoded.width(), decoded.height()),
            inner: decoded,
        })
    }

    pub fn width(&self) -> u32 {
        self.dim.0
    }

    pub fn height(&self) -> u32 {
        self.dim.1
    }

    pub(crate) fn resize_by_width(&self, nw: u32, filter: FilterType) -> Image {
        let (w, h) = self.dim;
        let nh = (h * nw) / w;
        Image {
            dim: (nw, nh),
            inner: self.inner.resize_exact(nw, nh, filter),
        }
    }

    pub(crate) fn resize_by_height(&self, nh: u32, filter: FilterType) -> Image {
        let (w, h) = self.dim;
        let nw = (w * nh) / h;
        Image {
            dim: (nw, nh),
            inner: self.inner.resize_exact(nw, nh, filter),
        }
    }

    pub fn is_landscape(&self) -> bool {
        self.width() > self.height()
    }

    pub fn thumbhash(&self) -> ThumbHash {
        let resized = if self.is_landscape() {
            self.resize_by_width(128, FilterType::CatmullRom)
        } else {
            self.resize_by_height(128, FilterType::CatmullRom)
        };

        let rgba = resized.inner.pixels().map(|p| p.2).collect::<Vec<_>>();

        thumbhash(&rgba, resized.width() as usize, resized.height() as usize)
    }
}

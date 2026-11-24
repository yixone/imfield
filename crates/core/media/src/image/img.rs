use std::io::Cursor;

use image::{DynamicImage, GenericImageView, imageops::FilterType};
use imfield_models::types::media::{PerceptualHash, ThumbHash};
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::image::{
    ImageError,
    hash::{a_hash, p_hash, thumbhash},
};

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

    pub fn resize_by_width(&self, nw: u32, filter: FilterType) -> Image {
        let (w, h) = self.dim;
        let nh = (h * nw) / w;
        Image {
            dim: (nw, nh),
            inner: self.inner.resize_exact(nw, nh, filter),
        }
    }

    pub fn resize_by_height(&self, nh: u32, filter: FilterType) -> Image {
        let (w, h) = self.dim;
        let nw = (w * nh) / h;
        Image {
            dim: (nw, nh),
            inner: self.inner.resize_exact(nw, nh, filter),
        }
    }

    pub fn resize(&self, nw: u32, nh: u32, filter: FilterType) -> Image {
        Image {
            dim: (nw, nh),
            inner: self.inner.resize_exact(nw, nh, filter),
        }
    }

    pub fn is_landscape(&self) -> bool {
        self.width() > self.height()
    }

    pub fn thumbhash(&self) -> ThumbHash {
        let rgba = self.inner.pixels().map(|p| p.2).collect::<Vec<_>>();
        thumbhash(&rgba, self.width() as usize, self.height() as usize)
    }

    pub fn a_hash(&self) -> PerceptualHash {
        let pixels = self.prepare_luma_matrix();
        a_hash(pixels)
    }

    pub fn p_hash(&self) -> PerceptualHash {
        let pixels = self.prepare_luma_matrix();
        p_hash(pixels)
    }

    fn prepare_luma_matrix<T, K>(&self) -> T
    where
        K: TryFrom<Vec<u8>>,
        T: TryFrom<Vec<K>>,
        T::Error: std::fmt::Debug,
        K::Error: std::fmt::Debug,
    {
        // FIXME: Replace unwrap with ImageError
        let luma = self.inner.to_luma8();
        luma.rows()
            .map(|i| i.map(|p| p.0[0]).collect::<Vec<_>>().try_into().unwrap())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }
}

use image::imageops::FilterType;

pub enum ResizeFilter {
    Nearest,
    Triangle,
    CatmullRom,
    Gaussian,
    Lanczos3,
}

impl From<ResizeFilter> for FilterType {
    fn from(f: ResizeFilter) -> Self {
        match f {
            ResizeFilter::Nearest => FilterType::Nearest,
            ResizeFilter::Triangle => FilterType::Triangle,
            ResizeFilter::CatmullRom => FilterType::CatmullRom,
            ResizeFilter::Gaussian => FilterType::Gaussian,
            ResizeFilter::Lanczos3 => FilterType::Lanczos3,
        }
    }
}

/// List of encoding formats
pub enum ImageFormat {
    /// An Image in PNG Format
    Png,
    /// An Image in JPEG Format
    Jpeg,
    /// An Image in WEBP Format with encode quality (0-100)
    WebP(usize),
}

impl From<ImageFormat> for image::ImageFormat {
    fn from(i: ImageFormat) -> Self {
        match i {
            ImageFormat::Png => image::ImageFormat::Png,
            ImageFormat::Jpeg => image::ImageFormat::Jpeg,
            ImageFormat::WebP { .. } => image::ImageFormat::WebP,
        }
    }
}

#[derive(Debug)]
pub enum ImageError {
    Io(std::io::Error),
    BackendError(image::ImageError),
}

impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for ImageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ImageError::Io(err) => Some(err),
            ImageError::BackendError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ImageError {
    fn from(err: std::io::Error) -> Self {
        ImageError::Io(err)
    }
}

impl From<image::ImageError> for ImageError {
    fn from(err: image::ImageError) -> Self {
        ImageError::BackendError(err)
    }
}

#[derive(Debug)]
pub enum VideoError {
    BinUnavailable,

    MissingStream,
    MissingMetadata,
    DecodeFailed,

    FileNotFound,
    FileAlreadyExists,

    DeserializeError(serde_json::Error),
    Io(std::io::Error),
}

impl std::fmt::Display for VideoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for VideoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            VideoError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for VideoError {
    fn from(err: serde_json::Error) -> Self {
        VideoError::DeserializeError(err)
    }
}

impl From<std::io::Error> for VideoError {
    fn from(err: std::io::Error) -> Self {
        VideoError::Io(err)
    }
}

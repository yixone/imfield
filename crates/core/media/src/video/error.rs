#[derive(Debug)]
pub enum VideoError {
    BinUnavailable,

    MissingStream,
    MissingMetadata,
    DecodeFailed,

    FileNotFound,

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
            VideoError::Io(error) => Some(error),
            _ => None,
        }
    }
}

// TODO: Rename to `VideoError`

#[derive(Debug)]
pub enum VideoToolError {
    BinUnavailable,
    OutputDecodeFailed,
    MissingVideoStream,
    Io(std::io::Error),
}

impl std::fmt::Display for VideoToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for VideoToolError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            VideoToolError::Io(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum VideoToolError {
    BinUnavailable,
    StdoutDecodeFailed,
}

impl std::fmt::Display for VideoToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for VideoToolError {}

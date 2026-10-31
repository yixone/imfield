pub enum StorageError {
    InvalidPath,
    NoStorageSpace,
    Other(Box<dyn std::error::Error + Send + 'static>),
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::Other(Box::new(err))
    }
}

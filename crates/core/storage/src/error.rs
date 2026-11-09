pub enum StorageError {
    InvalidPath,
    NoStorageSpace,
    Io(std::io::Error),
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::Io(err)
    }
}

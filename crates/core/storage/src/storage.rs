use std::path::PathBuf;

use imfield_storage_address::StoragePath;
use imfield_utils::futures::asyncify;

use crate::{
    StorageError,
    fs::create_parents,
    system::{FileSystemSpace, storage_stats},
};

pub struct Storage {
    root: PathBuf,
}

impl Storage {
    /// Creates a new [`Storage`]
    pub async fn new(root: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let root = root.into();
        create_parents(&root).await?;

        let storage = Storage { root };
        Ok(storage)
    }

    /// Resolves a logical storage path to its physical filesystem path
    pub fn realpath(&self, path: impl AsRef<StoragePath>) -> PathBuf {
        self.root.join(path.as_ref().as_str())
    }

    /// Returns disk usage statistics for the storage filesystem
    ///
    /// The statistics describe the filesystem containing the storage root
    pub async fn disk_usage(&self) -> Result<FileSystemSpace, StorageError> {
        let path = self.root.to_path_buf();
        let stats = asyncify(move || storage_stats(path)).await??;
        Ok(stats)
    }

    pub async fn ensure_enough_space(&self, required: u64) -> Result<(), StorageError> {
        if self.disk_usage().await?.bytes_free() <= required {
            return Err(StorageError::NoStorageSpace);
        }
        Ok(())
    }
}

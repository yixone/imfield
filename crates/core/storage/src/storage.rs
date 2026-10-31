use std::path::PathBuf;

use imfield_storage_address::StoragePath;
use imfield_utils::futures::asyncify;

use crate::{
    StorageError,
    files::metadata::FileMetadata,
    fs::{create_parents, remove_parents},
    system::{FileSystemSpace, rename_exclusive, storage_stats},
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

    /// Returns a [`StorageError::NoStorageSpace`] error if there is insufficient space in the storage
    #[inline]
    pub async fn ensure_enough_space(&self, required: u64) -> Result<(), StorageError> {
        if self.disk_usage().await?.bytes_free() <= required {
            return Err(StorageError::NoStorageSpace);
        }
        Ok(())
    }

    /// Returns metadata for a stored file
    ///
    /// The returned metadata includes the file length and modification time
    pub async fn meta(&self, path: impl AsRef<StoragePath>) -> Result<FileMetadata, StorageError> {
        let path = self.realpath(path);
        let meta = tokio::fs::metadata(&path).await?;

        let metadata = FileMetadata::new(meta.len(), meta.modified().ok());

        Ok(metadata)
    }

    /// Returns whether a file exists at the specified storage path
    ///
    /// Returns `false` if the path does not exists
    pub async fn exists(&self, path: impl AsRef<StoragePath>) -> Result<bool, StorageError> {
        let path = self.realpath(path);

        let exists = tokio::fs::try_exists(path).await?;

        Ok(exists)
    }

    /// Renames a stored file to another storage path
    ///
    /// The destination must not already exists. Missing parent directories of the
    /// destination are created automatically
    pub async fn rename(
        &self,
        path: impl AsRef<StoragePath>,
        dest: impl AsRef<StoragePath>,
    ) -> Result<(), StorageError> {
        let from = self.realpath(path);
        let to = self.realpath(dest);

        create_parents(&to).await?;
        asyncify(move || rename_exclusive(from, to)).await??;

        Ok(())
    }

    /// Removes a stored file
    ///
    /// Returns `true` if the file was removed and `false` if it did not exist
    ///
    /// Empty parent directories are removed after successful deletion
    pub async fn remove(&self, path: impl AsRef<StoragePath>) -> Result<bool, StorageError> {
        let path = self.realpath(path);

        match tokio::fs::remove_file(&path).await {
            Ok(_) => {
                remove_parents(self.root.as_path(), &path).await?;
                Ok(true)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(StorageError::Io(e)),
        }
    }
}

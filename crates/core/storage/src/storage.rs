use std::{io::SeekFrom, path::PathBuf};

use imfield_storage_address::StoragePath;
use imfield_utils::futures::asyncify;
use tokio::{
    fs::File,
    io::{AsyncSeekExt, BufReader, BufWriter},
};

use crate::{
    StorageError,
    files::{
        FileReader, FileWriter, ForeignUploader, LocalFile, StagedFile, metadata::FileMetadata,
    },
    fs::{self, create_parents, remove_parents},
    layout::StorageLayout,
    system::{FileSystemSpace, rename_exclusive, storage_stats},
};

/// Managed file storage
pub struct Storage {
    root: PathBuf,
}

impl Storage {
    /// Creates a new [`Storage`]
    pub async fn new(root: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let root = root.into();
        tokio::fs::create_dir_all(&root).await?;

        let storage = Storage { root };
        Ok(storage)
    }

    /// Resolves a logical storage path to its physical filesystem path
    pub fn realpath(&self, path: impl AsRef<StoragePath>) -> PathBuf {
        self.root.join(path.as_ref().as_str())
    }

    /// Creates a writer for a new staged file
    ///
    /// The target path must not already exist
    pub async fn create_writer(
        &self,
        path: impl AsRef<StoragePath>,
    ) -> Result<FileWriter, StorageError> {
        let staged = self.stage(path).await?;

        let file = File::create_new(&staged.temp_path).await?;
        let writer = BufWriter::with_capacity(32 * 1024, file);

        Ok(FileWriter::new(staged, writer))
    }

    /// Creates a foreign uploader for a new staged file
    ///
    /// The returned [`ForeignUploader`] provides a temporary filesystem path
    /// where an external process or another writer can create the file
    ///
    /// The target path must not already exists
    pub async fn create_uploader(
        &self,
        path: impl AsRef<StoragePath>,
    ) -> Result<ForeignUploader, StorageError> {
        let staged = self.stage(path).await?;

        Ok(ForeignUploader::new(staged))
    }

    /// Creates a staged file for the specified storage path
    ///
    /// The target path must not already exists
    async fn stage(&self, path: impl AsRef<StoragePath>) -> Result<StagedFile, StorageError> {
        let path = path.as_ref();

        let dest_path = self.realpath(path);
        if tokio::fs::try_exists(&dest_path).await? {
            return Err(StorageError::AlreadyExists);
        }

        let temp_path = {
            let path = StorageLayout::temp_path(path.clone())?;
            self.realpath(path)
        };

        fs::create_parents(&temp_path).await?;

        Ok(StagedFile::new(temp_path, dest_path, true))
    }

    /// Opens a locally stored file
    ///
    /// Returns a [`LocalFile`] containing the physical filesystem path of the
    /// stored file
    pub async fn get_local(
        &self,
        path: impl AsRef<StoragePath>,
    ) -> Result<LocalFile, StorageError> {
        let path = self.realpath(path);
        let meta = tokio::fs::metadata(&path).await?;

        Ok(LocalFile::new(path, meta.len()))
    }

    /// Opens a stored file for reading
    ///
    /// The reader starts at the beginning of the file and reads until EOF
    pub async fn get_reader(
        &self,
        path: impl AsRef<StoragePath>,
    ) -> Result<FileReader, StorageError> {
        self.get_reader_seek(path, 0, None).await
    }

    /// Opens a stored file for reading from a specified byte range
    ///
    /// `start` specifies the first byte to read and `end`, when specified,
    /// specifies the last byte to read inclusively
    ///
    /// For example, `start = 10` and `end = Some(19)` reads bytes `10..=19`
    ///
    /// The requested range must be within the file bounds
    pub async fn get_reader_seek(
        &self,
        path: impl AsRef<StoragePath>,
        start: u64,
        end: Option<u64>,
    ) -> Result<FileReader, StorageError> {
        let path = self.realpath(path);
        let meta = tokio::fs::metadata(&path).await?;

        if start > meta.len() {
            return Err(StorageError::InvalidSeekRange);
        }

        if let Some(end) = end
            && (end < start || end >= meta.len())
        {
            return Err(StorageError::InvalidSeekRange);
        }

        let mut file = File::open(&path).await?;
        file.seek(SeekFrom::Start(start)).await?;

        let reader = BufReader::with_capacity(32 * 1024, file);

        Ok(match end {
            Some(end) => {
                let to_read = end.saturating_sub(start) + 1;
                FileReader::new(reader, to_read)
            }
            None => FileReader::new(reader, meta.len() - start),
        })
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
        if self.disk_usage().await?.bytes_free() < required {
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

        let from_cloned = from.clone();
        let to = self.realpath(dest);
        create_parents(&to).await?;

        asyncify(move || rename_exclusive(from_cloned, to)).await??;
        remove_parents(self.root.as_path(), &from).await?;

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

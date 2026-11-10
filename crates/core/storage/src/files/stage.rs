use std::path::PathBuf;

use imfield_utils::futures::asyncify;

use crate::{StorageError, fs, system::rename_exclusive};

/// A file staged for atomic publication
///
/// A staged file is written to a temporary path and can be published
/// at its target path using [`StagedFile::commit`]. Dropping an uncommitted
/// staged file removes its temporary file
pub struct StagedFile {
    pub(crate) temp_path: PathBuf,
    pub(crate) dest_path: PathBuf,
    need_cleanup: bool,
}

impl StagedFile {
    /// Creates a new [`StagedFile`]
    pub fn new(temp: PathBuf, dest: PathBuf, need_cleanup: bool) -> Self {
        Self {
            temp_path: temp,
            dest_path: dest,
            need_cleanup,
        }
    }

    /// Atomically publishes the staged file at its target path
    ///
    /// The operation fails if the target already exists. On failure, the
    /// staged file remains owned by the operation and is cleaned up on drop
    pub async fn commit(mut self) -> Result<(), StorageError> {
        let (from, to) = (self.temp_path.clone(), self.dest_path.clone());
        fs::create_parents(&to).await?;

        asyncify(move || rename_exclusive(from, to)).await??;
        self.need_cleanup = false;

        Ok(())
    }

    /// Aborts the staged file and removes its temporary file
    ///
    /// Aborting an already removed temporary file succeeds without error
    pub async fn abort(mut self) -> Result<(), StorageError> {
        self.need_cleanup = false;
        match tokio::fs::remove_file(&self.temp_path).await {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

impl Drop for StagedFile {
    fn drop(&mut self) {
        if !self.need_cleanup {
            return;
        }

        let _ = std::fs::remove_file(&self.temp_path);
    }
}

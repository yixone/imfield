use std::{
    path::{Path, PathBuf},
    pin::pin,
};

use futures::TryStreamExt;
use tokio::{
    fs::File,
    io::{AsyncRead, AsyncSeek, AsyncWriteExt, BufReader, BufWriter},
};
use tokio_util::io::ReaderStream;

use crate::{StorageError, files::StagedFile};

/// Writes data to a [`StagedFile`]
pub struct FileWriter {
    staged: StagedFile,

    writer: BufWriter<File>,
    bytes_written: u64,
}

impl FileWriter {
    /// Creates a new [`FileWriter`]
    pub fn new(staged: StagedFile, writer: BufWriter<File>) -> Self {
        Self {
            staged,
            writer,
            bytes_written: 0,
        }
    }

    /// Writes data to the staged file
    pub async fn write(&mut self, data: impl AsRef<[u8]>) -> Result<(), StorageError> {
        let bytes = data.as_ref();

        self.writer.write_all(bytes).await?;
        self.bytes_written += bytes.len() as u64;

        Ok(())
    }

    /// Writers data from the asynchronous readet into the staged file
    ///
    /// The reader is consumed until EOF
    pub async fn write_reader<R>(&mut self, reader: R) -> Result<(), StorageError>
    where
        R: AsyncRead + Unpin,
    {
        self.write_reader_callback(reader, |_| Ok(())).await
    }

    /// Writers data from the asynchronous readet into the staged file
    ///
    /// The callback is called for each chunk from the reader, before
    /// the chunk is written to the staged file
    ///
    /// The operation stops if reading, writing, or the callback returns an error
    pub async fn write_reader_callback<R, F, E>(
        &mut self,
        reader: R,
        mut callback: F,
    ) -> std::result::Result<(), E>
    where
        R: AsyncRead + Unpin,
        F: FnMut(&[u8]) -> Result<(), StorageError>,
        E: From<StorageError>,
    {
        let mut reader = ReaderStream::with_capacity(reader, 32 * 1024);
        while let Some(chunk) = reader.try_next().await.map_err(StorageError::from)? {
            callback(&chunk)?;
            self.write(chunk).await?;
        }

        Ok(())
    }

    /// Atomically publishes the staged file at its target path
    pub async fn commit(mut self) -> Result<u64, StorageError> {
        self.writer.flush().await?;
        self.staged.commit().await?;

        Ok(self.bytes_written)
    }

    /// Aborts the writer and removes its temporary file
    pub async fn abort(self) -> Result<(), StorageError> {
        self.staged.abort().await
    }

    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }
}

/// Provides a temporary path for uploading a staged file using an external
/// process or another foreign writer
pub struct ForeignUploader {
    staged: StagedFile,
}

impl ForeignUploader {
    /// Creates a new [`ForeignUploader`]
    pub fn new(staged: StagedFile) -> Self {
        Self { staged }
    }

    /// Returns the temporary path where the foreign writer should store data
    pub fn path(&self) -> &Path {
        &self.staged.temp_path
    }

    /// Finishes the upload and commits the staged file
    pub async fn finish(self) -> Result<u64, StorageError> {
        let meta = tokio::fs::metadata(&self.staged.temp_path).await?;
        let written = meta.len();

        self.staged.commit().await?;

        Ok(written)
    }

    /// Aborts the uploader and removes its temporary file
    pub async fn abort(self) -> Result<(), StorageError> {
        self.staged.abort().await
    }
}

/// Provides access to a locally stored file
pub struct LocalFile {
    path: PathBuf,
    len: u64,
}

impl LocalFile {
    /// Creates a new [`LocalFile`]
    pub fn new(path: PathBuf, len: u64) -> Self {
        Self { path, len }
    }

    /// Returns the local filesystem path of this file
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the length of the file
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Returns `true` if the file is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Wrapper for a stored file reader
pub struct FileReader {
    reader: BufReader<File>,
    len: u64,
}

impl FileReader {
    /// Creates a new [`FileReader`]
    pub fn new(reader: BufReader<File>, len: u64) -> Self {
        Self { reader, len }
    }

    /// Returns the length of the file when the reader was created
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Returns `true` if the file is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl AsyncRead for FileReader {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        pin!(&mut self.reader).poll_read(cx, buf)
    }
}

impl AsyncSeek for FileReader {
    fn poll_complete(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<u64>> {
        pin!(&mut self.reader).poll_complete(cx)
    }

    fn start_seek(
        mut self: std::pin::Pin<&mut Self>,
        position: std::io::SeekFrom,
    ) -> std::io::Result<()> {
        pin!(&mut self.reader).start_seek(position)
    }
}

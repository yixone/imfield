#[async_trait::async_trait]
pub trait FilesDatabase: Send + Sync {}

#[async_trait::async_trait]
pub trait FilesDatabaseExt: FilesDatabase {}

impl<T> FilesDatabaseExt for T where T: FilesDatabase {}

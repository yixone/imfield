use std::sync::Arc;

pub mod assets;
pub mod files;

pub use assets::{AssetsDatabase, AssetsDatabaseExt};
pub use files::{FilesDatabase, FilesDatabaseExt};

pub struct RepositoriesContext {
    pub assets: Arc<dyn AssetsDatabase>,
    pub files: Arc<dyn FilesDatabase>,
}

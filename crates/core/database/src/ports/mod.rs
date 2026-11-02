use std::sync::Arc;

pub mod assets;
pub mod files;

pub struct RepositoriesContext {
    pub assets: Arc<dyn assets::AssetsDatabase>,
    pub files: Arc<dyn files::FilesDatabase>,
}

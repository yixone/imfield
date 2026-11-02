use imfield_models::models::{Asset, File, FileContainer};

/// An [`Asset`] together with its associated [`FileContainerView`]
pub type AssetView = (Asset, FileContainerView);

pub type FileContainerView = (FileContainer, Vec<File>);

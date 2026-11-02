use imfield_models::entities::{Asset, File, FileContainer};

/// An [`Asset`] together with its associated [`FileContainerView`]
pub type AssetView = (Asset, FileContainerView);

pub type FileContainerView = (FileContainer, Vec<File>);

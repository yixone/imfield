use imfield_domains::types::{files::FileVariant, id::AssetId};
use imfield_storage_address::{StoragePath, path::PATH_SEP_CHAR};

use crate::StorageError;

const MEDIA_DIR: &str = "media";

const TEMP_SUFFIX_LEN: usize = 6;

/// Generates storage paths for AssetShelf files
pub struct StorageLayout;

impl StorageLayout {
    /// Returns the storage path for a media variant
    pub fn media_path(id: AssetId, variant: FileVariant) -> StoragePath {
        StoragePath::new(MEDIA_DIR)
            .push(shard(id.to_string(), 2))
            .push(variant.as_str())
    }

    /// Creates a temporary path based on the specified storage path
    ///
    /// The original file name is prefixed with `~` and suffixed with random
    /// identifier to produce an unique temporary file name
    pub fn temp_path(mut p: StoragePath) -> Result<StoragePath, StorageError> {
        let suffix = imfield_utils::rand::random_b64(TEMP_SUFFIX_LEN);
        let temp_name = p
            .file_name()
            .map(|p| format!("~{}-{}", p, suffix))
            .ok_or(StorageError::InvalidPath)?;

        p.set_file_name(temp_name);
        Ok(p)
    }
}

/// Applies path sharding, transforming: `abcdef` into `ab/cd/abcdef`
///
/// The input value must contain only ASCII characters
fn shard(value: impl AsRef<str>, steps: usize) -> String {
    let path = value.as_ref();

    let mut res = String::with_capacity(path.len() + steps * 3);

    for i in 0..steps {
        let idx = 2 * i;
        if idx + 2 > path.len() {
            break;
        }
        res.push_str(&path[idx..idx + 2]);
        res.push(PATH_SEP_CHAR);
    }

    res.push_str(path);
    res
}

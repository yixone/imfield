use imfield_di::DataContext;
use imfield_models::entities::file::FileResource;
use tokio_util::sync::CancellationToken;

use crate::{JobContext, error::JobError};

pub async fn cleanup_deleted(ctx: &JobContext, cancel: CancellationToken) -> Result<(), JobError> {
    todo!()
}

async fn delete_file(ctx: &DataContext, file: FileResource) -> Result<(), JobError> {
    // Delete the sidecars
    for s in file.sidecars {
        ctx.storage.remove(s.path()).await?;
        ctx.db
            .files
            .delete_sidecar(s.file_id(), s.sidecar_type())
            .await?;
    }

    // Delete the original file
    ctx.storage.remove(file.file.path()).await?;
    ctx.db.files.delete_file(file.file.id()).await?;

    Ok(())
}

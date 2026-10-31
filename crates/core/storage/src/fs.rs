use std::path::Path;

/// Creates all parent directories for the specified path
pub async fn create_parents(path: impl AsRef<Path>) -> std::io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    Ok(())
}

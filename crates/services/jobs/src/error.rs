#[derive(Debug)]
pub enum JobError {
    Cancelled,
    Domain(imfield_models::DomainError),
    Database(imfield_database::DatabaseError),
    Storage(imfield_storage::StorageError),
    Image(imfield_media::image::ImageError),
    Video(imfield_media::video::VideoError),
    ProcessingTimeout,
    Other(Box<dyn std::error::Error + Send>),
}

impl std::fmt::Display for JobError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for JobError {}

impl From<imfield_models::DomainError> for JobError {
    fn from(e: imfield_models::DomainError) -> Self {
        JobError::Domain(e)
    }
}

impl From<std::io::Error> for JobError {
    fn from(e: std::io::Error) -> Self {
        JobError::Other(Box::new(e))
    }
}

impl From<imfield_database::DatabaseError> for JobError {
    fn from(e: imfield_database::DatabaseError) -> Self {
        JobError::Database(e)
    }
}

impl From<imfield_storage::StorageError> for JobError {
    fn from(e: imfield_storage::StorageError) -> Self {
        JobError::Storage(e)
    }
}

impl From<imfield_media::image::ImageError> for JobError {
    fn from(e: imfield_media::image::ImageError) -> Self {
        JobError::Image(e)
    }
}

impl From<imfield_media::video::VideoError> for JobError {
    fn from(e: imfield_media::video::VideoError) -> Self {
        JobError::Video(e)
    }
}

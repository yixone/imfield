#[derive(Debug)]
pub enum JobError {
    Cancelled,
    Domain(imfield_models::DomainError),
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

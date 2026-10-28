/// Application domain errors
#[derive(Debug)]
pub enum DomainError {
    /// Entity not found / does not exist
    NotFound,
    /// The entity already exists, and conflict is impermissible
    AlreadyExists,
    /// Input data violates constraints
    BadInput,
    /// ...
    RelationViolation,
    /// Custom error
    Custom(Box<dyn std::error::Error + Send + 'static>),
}

impl DomainError {
    /// Creates a new [`DomainError::Custom`] with an erased error type
    pub fn custom<E>(err: E) -> Self
    where
        E: std::error::Error + Send + 'static,
    {
        Self::Custom(Box::new(err))
    }
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for DomainError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DomainError::Custom(e) => Some(&**e),
            _ => None,
        }
    }
}

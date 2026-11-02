use imfield_models::DomainError;

pub enum DatabaseError {
    InvalidPagination,
    Domain(DomainError),
}

impl From<DomainError> for DatabaseError {
    fn from(e: DomainError) -> Self {
        DatabaseError::Domain(e)
    }
}

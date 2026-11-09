use imfield_models::DomainError;

pub enum DatabaseError {
    InvalidPagination,
    Domain(DomainError),
    Io(std::io::Error),
    Other(Box<dyn std::error::Error + Send + 'static>),
}

impl From<DomainError> for DatabaseError {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::Other(error) => DatabaseError::Other(error),
            _ => DatabaseError::Domain(e),
        }
    }
}

pub trait ToDatabaseError<T> {
    fn to_db_error(self) -> Result<T, DatabaseError>;
}

impl<T, E> ToDatabaseError<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + 'static,
{
    #[track_caller]
    fn to_db_error(self) -> Result<T, DatabaseError> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(DatabaseError::Other(Box::new(e))),
        }
    }
}

use imfield_domains::DomainError;

/// A closure that validates and updates an entity
///
/// Returning an error aborts the update
pub type UpdateFn<T> = dyn (FnOnce(&mut T) -> Result<(), DomainError>) + Send;

/// A closure that validates an entity before deletion
///
/// Returning an error aborts the deletion
pub type DeleteFn<T> = dyn (FnOnce(&T) -> Result<(), DomainError>) + Send;

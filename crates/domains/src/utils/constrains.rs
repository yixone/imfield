use crate::DomainError;

pub fn ensure_str_len<S>(str: S, len: usize) -> Result<(), DomainError>
where
    S: AsRef<str>,
{
    if str.as_ref().len() > len {
        return Err(DomainError::BadInput);
    }

    Ok(())
}

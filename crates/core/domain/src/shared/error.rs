use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("database error: {0}")]
    DatabaseError(#[source] anyhow::Error),

    #[error("password error: {0}")]
    PasswordManagerError(#[source] anyhow::Error),

    #[error("password not match")]
    PasswordNotMatch,
}

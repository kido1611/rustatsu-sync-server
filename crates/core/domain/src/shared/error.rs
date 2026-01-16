use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("database error: {0}")]
    DatabaseError(#[source] anyhow::Error),

    #[error("password error: {0}")]
    PasswordManagerError(#[source] anyhow::Error),

    #[error("password not match")]
    PasswordNotMatch,

    #[error("hash token error: {0}")]
    HashTokenError(#[source] anyhow::Error),

    #[error("auth token error: {0}")]
    AuthTokenError(#[source] anyhow::Error),
}

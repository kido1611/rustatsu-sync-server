#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Auth token is missing")]
    TokenMissing(anyhow::Error),
    #[error("Jwt error")]
    JwtError(jsonwebtoken::errors::Error),
    #[error("Password error")]
    PasswordError(argon2::password_hash::Error),
    #[error("Unauthenticated")]
    Unauthenticated,
    #[error("User not found")]
    UserNotFound,
    #[error("Incorrect credential")]
    IncorrectCredential,
}

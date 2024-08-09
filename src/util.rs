use axum::http::StatusCode;
use axum_thiserror::ErrorStatus;

#[derive(thiserror::Error, ErrorStatus)]
pub enum MangaError {
    #[error("Manga is missing")]
    #[status(StatusCode::NOT_FOUND)]
    Missing(#[source] anyhow::Error),

    #[error("Something went wrong")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    UnexpectedError(#[from] anyhow::Error),

    #[error("Invalid Credential")]
    #[status(StatusCode::UNAUTHORIZED)]
    InvalidCredential(#[source] anyhow::Error),

    #[error("Content Equal")]
    #[status(StatusCode::NO_CONTENT)]
    ContentEqual(#[source] anyhow::Error),
}

#[derive(thiserror::Error, ErrorStatus)]
pub enum AuthError {
    #[error("User is missing")]
    #[status(StatusCode::BAD_REQUEST)]
    UserMissing(#[source] anyhow::Error),
    #[error("Incorrect email format")]
    #[status(StatusCode::BAD_REQUEST)]
    ValidationEmailInvalid(#[source] anyhow::Error),
    #[error("Email length must be between 3 and 128 characters")]
    #[status(StatusCode::BAD_REQUEST)]
    ValidationEmailLength(#[source] anyhow::Error),
    #[error("Password length must be between 8 and 128 characters")]
    #[status(StatusCode::BAD_REQUEST)]
    ValidationPasswordLength(#[source] anyhow::Error),
    #[error("Invalid credential")]
    #[status(StatusCode::BAD_REQUEST)]
    InvalidPassword(#[source] anyhow::Error),
    #[error("Something went wrong")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("Auth header is missing")]
    #[status(StatusCode::UNAUTHORIZED)]
    EmptyAuthHeader(#[source] anyhow::Error),
    #[error("Auth token is missing")]
    #[status(StatusCode::UNAUTHORIZED)]
    EmptyAuthToken(#[source] anyhow::Error),
    #[error("Invalid Credential")]
    #[status(StatusCode::UNAUTHORIZED)]
    InvalidCredential(#[source] anyhow::Error),
}

impl std::fmt::Debug for MangaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Debug for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }

    Ok(())
}

use axum::{http::StatusCode, response::IntoResponse};
use core_application::shared::error::ApplicationError;
use validator::ValidationErrors;

#[derive(thiserror::Error)]
pub enum Error {
    #[error("error from jwt: {0}")]
    JwtError(
        #[source]
        #[from]
        jsonwebtoken::errors::Error,
    ),

    #[error("Validation error: {0}")]
    Validation(
        #[source]
        #[from]
        ValidationErrors,
    ),

    #[error("unexpected error: {0}")]
    UnexpectedError(#[source] anyhow::Error),

    #[error("application error: {0}")]
    ApplicationError(
        #[source]
        #[from]
        ApplicationError,
    ),

    #[error("{0} not found")]
    ResourceNotFound(String),

    #[error("unauthorized")]
    Unauthorized,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("error: {:?}", self);

        match self {
            Error::Validation(validation_error) => {
                (StatusCode::BAD_REQUEST, validation_error.to_string()).into_response()
            }
            Error::ApplicationError(application_error) => match application_error {
                ApplicationError::DomainError(domain_error) => match domain_error {
                    core_domain::shared::error::DomainError::DatabaseError(_) => {
                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    }
                    core_domain::shared::error::DomainError::PasswordManagerError(_) => {
                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    }
                    core_domain::shared::error::DomainError::PasswordNotMatch => {
                        StatusCode::UNAUTHORIZED.into_response()
                    }
                    core_domain::shared::error::DomainError::HashTokenError(_) => {
                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    }
                },
                ApplicationError::RegisterIsForbidden => StatusCode::UNAUTHORIZED.into_response(),
                ApplicationError::ResourceNotFound(message) => {
                    (StatusCode::NOT_FOUND, message).into_response()
                }
            },
            Error::ResourceNotFound(message) => {
                (StatusCode::NOT_FOUND, format!("{} not found", message)).into_response()
            }
            Error::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
            Error::JwtError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

impl std::fmt::Debug for Error {
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

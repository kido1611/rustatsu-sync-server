use axum::{http::StatusCode, response::IntoResponse};
use validator::ValidationErrors;

use crate::{auth::error::AuthError, db::error::DatabaseError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Database error")]
    Database(DatabaseError),

    #[error("Auth error")]
    Auth(AuthError),

    #[error("Validation error")]
    Validation(ValidationErrors),

    #[error("Other error: {0}")]
    Other(anyhow::Error),
}

impl From<DatabaseError> for Error {
    fn from(value: DatabaseError) -> Self {
        Self::Database(value)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::Database(database_error) => match database_error {
                DatabaseError::DatabaseError(error) => {
                    eprintln!("{}", error);

                    tracing::error!(err.msg = %error, err.details=?error, "Database Error");

                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
                DatabaseError::NotFound => StatusCode::NOT_FOUND.into_response(),
            },
            Error::Auth(auth_error) => match auth_error {
                AuthError::TokenMissing(error) => {
                    (StatusCode::UNAUTHORIZED, error.to_string()).into_response()
                }
                AuthError::JwtError(error) => {
                    tracing::error!(err.msg = %error, err.details=?error, "JWT Error");

                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
                AuthError::Unauthenticated => StatusCode::UNAUTHORIZED.into_response(),
                AuthError::PasswordError(error) => {
                    tracing::error!(err.msg = %error, err.details=?error, "Password Hash Error");

                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
                AuthError::UserNotFound => StatusCode::UNAUTHORIZED.into_response(),
                AuthError::IncorrectCredential => StatusCode::UNAUTHORIZED.into_response(),
            },
            Error::Other(error) => {
                tracing::error!(err.msg = %error, err.details=?error, "Other Error");

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            Error::Validation(validation_error) => {
                tracing::error!(err.msg = %validation_error, err.details=?validation_error, "Validation Error");

                (StatusCode::BAD_REQUEST, validation_error.to_string()).into_response()
            }
        }
    }
}

// impl std::fmt::Debug for Error {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         error_chain_fmt(self, f)
//     }
// }
//
// pub fn error_chain_fmt(
//     e: &impl std::error::Error,
//     f: &mut std::fmt::Formatter<'_>,
// ) -> std::fmt::Result {
//     writeln!(f, "{}\n", e)?;
//     let mut current = e.source();
//     while let Some(cause) = current {
//         writeln!(f, "Caused by:\n\t{}", cause)?;
//         current = cause.source();
//     }
//
//     Ok(())
// }

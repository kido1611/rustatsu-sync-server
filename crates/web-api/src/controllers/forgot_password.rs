use std::borrow::Cow;

use axum::{extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use validator::{Validate, ValidateEmail, ValidateLength, ValidationError, ValidationErrors};

use crate::{error::Error, state::SharedAppState};

#[derive(Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

impl Validate for ForgotPasswordRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if !self.email.validate_email() {
            errors.add(
                "email",
                ValidationError::new("email_email")
                    .with_message(Cow::from("Incorrect email format")),
            );
        }
        if !self.email.validate_length(Some(1), Some(100), None) {
            errors.add(
                "email",
                ValidationError::new("email_length")
                    .with_message(Cow::from("Email length must be between 1 and 100")),
            );
        }

        if !errors.errors().is_empty() {
            return Err(errors);
        }

        Ok(())
    }
}

#[tracing::instrument(name = "request::forgot_password", skip_all)]
pub async fn forgot_password(
    State(app_state): State<SharedAppState>,
    axum::extract::Json(request): axum::extract::Json<ForgotPasswordRequest>,
) -> Result<impl IntoResponse, Error> {
    request.validate().map_err(Error::Validation)?;

    app_state
        .request_reset_password_use_case
        .execute(&request.email, &app_state.config.application.get_base_url())
        .await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

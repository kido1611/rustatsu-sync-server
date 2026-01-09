use std::borrow::Cow;

use axum::{Json, extract::State};
use core_application::user::model::UserResetPasswordInput;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use validator::{Validate, ValidateLength, ValidationError, ValidationErrors};

use crate::{error::Error, state::SharedAppState};

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub reset_token: SecretString,
    pub password: SecretString,
}

impl Validate for ResetPasswordRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        let mut errors = ValidationErrors::new();

        let password = self.password.expose_secret();
        if !password.validate_length(Some(8), Some(32), None) {
            errors.add(
                "password",
                ValidationError::new("password_length")
                    .with_message(Cow::from("Password length must be between 8 and 32")),
            );
        }

        if !errors.errors().is_empty() {
            return Err(errors);
        }

        Ok(())
    }
}

#[tracing::instrument(name = "request::reset_password", skip_all, fields(reset_token = %request.reset_token.expose_secret()))]
pub async fn reset_password(
    State(app_state): State<SharedAppState>,
    Json(request): Json<ResetPasswordRequest>,
) -> Result<String, Error> {
    request.validate().map_err(Error::Validation)?;

    app_state
        .reset_user_password_use_case
        .execute(UserResetPasswordInput {
            token: request.reset_token,
            password: request.password,
        })
        .await?;

    Ok("Password has been reset successfully".to_string())
}

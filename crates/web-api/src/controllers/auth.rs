use std::borrow::Cow;

use anyhow::Context;
use axum::{Json, extract::State};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidateEmail, ValidateLength, ValidationError, ValidationErrors};

use crate::{
    auth::encode_jwt, error::Error, state::SharedAppState, telemetry::spawn_blocking_with_tracing,
};

#[derive(Deserialize)]
pub struct AuthRequest {
    pub email: String,
    pub password: SecretString,
}

impl Validate for AuthRequest {
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

#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
}

#[tracing::instrument(name = "request::auth", skip_all)]
pub async fn store(
    State(app_state): State<SharedAppState>,
    axum::extract::Json(request): axum::extract::Json<AuthRequest>,
) -> Result<Json<AuthResponse>, Error> {
    request.validate().map_err(Error::Validation)?;

    let user_dto = app_state
        .check_or_create_user_usecase
        .execute(core_application::user::model::UserInput {
            email: request.email,
            password: request.password,
            nickname: None,
        })
        .await?;

    let token = spawn_blocking_with_tracing(move || encode_jwt(user_dto.id, &app_state.config.jwt))
        .await
        .context("encode jwt")
        .map_err(Error::UnexpectedError)??;

    let token = AuthResponse {
        token: token.to_string(),
    };

    Ok(Json(token))
}

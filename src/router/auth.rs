use anyhow::Context;
use axum::{extract::State, Json};
use secrecy::{ExposeSecret, Secret};
use sqlx::MySqlPool;
use validator::{ValidateEmail, ValidateLength};

use crate::{
    authorization::{compute_password_hash, create_token, verify_password_hash, User},
    startup::AppState,
    telemetry::spawn_blocking_with_tracing,
    util::AuthError,
};

#[derive(serde::Deserialize, Debug)]
pub struct AuthForm {
    pub email: String,
    pub password: Secret<String>,
}

impl AuthForm {
    pub fn validate(&self) -> Result<(), AuthError> {
        let email = self.email.clone();

        if !email.validate_email() {
            return Err(AuthError::ValidationEmailInvalid(anyhow::anyhow!(
                "Incorrect email format"
            )));
        }

        if !email.validate_length(Some(3), Some(128), None) {
            return Err(AuthError::ValidationEmailLength(anyhow::anyhow!(
                "Email length must be between 3 to 128 characters"
            )));
        }

        let password = self.password.expose_secret();
        if !password.validate_length(Some(8), Some(128), None) {
            return Err(AuthError::ValidationPasswordLength(anyhow::anyhow!(
                "Password length must be between 8 to 128 characters"
            )));
        }

        Ok(())
    }
}

#[derive(serde::Serialize)]
pub struct AuthResult {
    token: String,
}

#[tracing::instrument(name = "Authentication", skip(app_state, form), fields(form.email))]
pub async fn auth(
    State(app_state): State<AppState>,
    axum::extract::Json(form): axum::extract::Json<AuthForm>,
) -> Result<Json<AuthResult>, AuthError> {
    // Validation
    form.validate()?;

    // get or create user
    let (user, user_password) = match get_or_create_user(
        &app_state.pool,
        app_state.config.application.allow_registration,
        form.email,
        form.password.clone(),
    )
    .await
    .map_err(|e| AuthError::UnexpectedError(e.into()))?
    {
        Some(u) => u,
        None => return Err(AuthError::UserMissing(anyhow::anyhow!("User not found"))),
    };

    // verify password
    spawn_blocking_with_tracing(move || verify_password_hash(user_password, form.password))
        .await
        .context("Failed when verifying password")
        .map_err(AuthError::UnexpectedError)?
        .map_err(AuthError::InvalidPassword)?;

    // Create token
    let token = spawn_blocking_with_tracing(move || create_token(user, app_state.config.jwt))
        .await
        .context("Failed generating JWT token")
        .map_err(AuthError::UnexpectedError)??;

    Ok(Json(AuthResult { token }))
}

#[tracing::instrument(name = "get user", skip(pool, allow_registration, password))]
async fn get_or_create_user(
    pool: &MySqlPool,
    allow_registration: bool,
    email: String,
    password: Secret<String>,
) -> Result<Option<(User, Secret<String>)>, anyhow::Error> {
    let user_row = sqlx::query!(
        r#"
        SELECT id, email, nickname, password
        FROM users
        where email = ?
        LIMIT 1
        "#,
        email
    )
    .fetch_optional(pool)
    .await
    .context("Failed when retrieve user")?
    .map(|row| (row.id, row.email, row.nickname, Secret::from(row.password)));

    if user_row.is_some() {
        let user_row = user_row.unwrap();

        return Ok(Some((
            User {
                id: user_row.0,
                email: user_row.1,
                nickname: user_row.2,
            },
            user_row.3,
        )));
    }

    if !allow_registration {
        return Ok(None);
    }

    let new_user = create_user(pool, email, password)
        .await
        .map_err(|e| AuthError::UnexpectedError(e.into()))?;
    Ok(Some(new_user))
}

#[tracing::instrument(name = "create a new user", skip(pool, password))]
async fn create_user(
    pool: &MySqlPool,
    email: String,
    password: Secret<String>,
) -> Result<(User, Secret<String>), AuthError> {
    let password_hash = spawn_blocking_with_tracing(move || compute_password_hash(password))
        .await
        .context("Failed calculating password hash")?
        .map_err(|e| AuthError::UnexpectedError(e.into()))?;

    let user_id = sqlx::query!(
        r#"
        INSERT INTO users
            (email, password, nickname, favourites_sync_timestamp, history_sync_timestamp)
        VALUES
            ( ? , ? , NULL, NULL, NULL );
        "#,
        email.clone(),
        password_hash.expose_secret()
    )
    .execute(pool)
    .await
    .map_err(|e| AuthError::UnexpectedError(e.into()))?
    .last_insert_id();

    Ok((
        User {
            id: i64::try_from(user_id).unwrap(),
            email,
            nickname: None,
        },
        password_hash,
    ))
}

use anyhow::Context;
use argon2::{
    password_hash::SaltString, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
};
use axum::{debug_handler, extract::State, Json};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use secrecy::{ExposeSecret, Secret};
use sqlx::{Execute, Executor, MySqlPool};

use crate::{telemetry::spawn_blocking_with_tracing, util::AuthError};

#[derive(serde::Deserialize, Debug)]
pub struct AuthForm {
    pub email: String,
    pub password: Secret<String>,
}

#[derive(serde::Serialize)]
pub struct AuthResult {
    token: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Claims {
    user_id: i64,
    aud: String,
    iss: String,
    exp: usize,
    iat: usize,
}

pub struct User {
    id: i64,
    email: String,
    nickname: Option<String>,
}

#[tracing::instrument(name = "Authentication", skip(pool, form), fields(form.email))]
#[debug_handler]
pub async fn auth(
    State(pool): State<MySqlPool>,
    axum::extract::Json(form): axum::extract::Json<AuthForm>,
) -> Result<Json<AuthResult>, AuthError> {
    // TODO: get from config

    // get or create user
    let (user, user_password) =
        match get_or_create_user(&pool, true, form.email, form.password.clone())
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
        .map_err(AuthError::UnexpectedError)??;

    let token = create_token(user).await?;

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
    .map_err(|e| {
        dbg!(&e);
        AuthError::UnexpectedError(e.into())
    })?
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

#[tracing::instrument(name = "calculate password hash", skip(password))]
fn compute_password_hash(password: Secret<String>) -> Result<Secret<String>, anyhow::Error> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    )
    .hash_password(password.expose_secret().as_bytes(), &salt)?
    .to_string();

    Ok(Secret::new(password_hash))
}

#[tracing::instrument(name = "verify password", skip_all)]
fn verify_password_hash(
    expected_password: Secret<String>,
    password: Secret<String>,
) -> Result<(), AuthError> {
    let expected_password_hash = PasswordHash::new(expected_password.expose_secret())
        .context("Failed to parse bash in PHC string format.")
        .map_err(AuthError::UnexpectedError)?;

    Argon2::default()
        .verify_password(password.expose_secret().as_bytes(), &expected_password_hash)
        .context("Invalid password")
        .map_err(AuthError::InvalidPassword)
}

#[tracing::instrument(name = "Create JWT token", skip(user), fields(user.id))]
async fn create_token(user: User) -> Result<String, AuthError> {
    // TODO: use config
    let secret = "my-secret-key".to_string();
    let iss = "http://localhost:8080/".to_string();
    let aud = "http://localhost:8080/resource".to_string();
    let now = Utc::now();
    let expire: chrono::TimeDelta = Duration::hours(24);
    let exp: usize = (now + expire).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claim = Claims {
        user_id: user.id,
        aud,
        iss,
        iat,
        exp,
    };

    let result = encode(
        &Header::default(),
        &claim,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .context("Failed wncoding token")
    .map_err(AuthError::UnexpectedError)?;

    Ok(result)
}

// pub enum AuthError {
// UnexpectedError(#[source] anyhow::Error),
// }

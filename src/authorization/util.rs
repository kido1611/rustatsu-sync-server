use anyhow::Context;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use secrecy::ExposeSecret;
use sqlx::MySqlPool;

use crate::{configuration::Jwt, error::ApiError};

#[derive(Clone, Debug)]
pub struct UserId(pub i64);

impl UserId {
    pub async fn to_user(&self, pool: &MySqlPool) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
                SELECT id, email, nickname
                FROM users
                WHERE id = ?
            "#,
            self.0
        )
        .fetch_optional(pool)
        .await
    }
}

#[derive(serde::Serialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub nickname: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Claim {
    pub user_id: i64,
    pub aud: String,
    pub iss: String,
    pub exp: usize,
    pub iat: usize,
}

#[tracing::instrument(name = "create jwt token", skip(user, jwt), fields(user.id))]
pub fn create_token(user: User, jwt: Jwt) -> Result<String, ApiError> {
    let now = Utc::now();
    let expire: chrono::TimeDelta = Duration::hours(24);
    let exp: usize = (now + expire).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claim = Claim {
        user_id: user.id,
        aud: jwt.aud.expose_secret().to_string(),
        iss: jwt.iss.expose_secret().to_string(),
        iat,
        exp,
    };

    let result = encode(
        &Header::default(),
        &claim,
        &EncodingKey::from_secret(jwt.secret.expose_secret().as_bytes()),
    )
    .context("Failed wncoding token")
    .map_err(ApiError::UnexpectedError)?;

    Ok(result)
}

#[tracing::instrument(name = "decode jwt token", skip(jwt_token, jwt))]
pub fn decode_jwt(jwt_token: String, jwt: Jwt) -> Result<TokenData<Claim>, ApiError> {
    let mut validation = Validation::default();
    validation.set_issuer(&[jwt.iss.expose_secret()]);
    validation.set_audience(&[jwt.aud.expose_secret()]);

    let result = decode::<Claim>(
        &jwt_token,
        &DecodingKey::from_secret(jwt.secret.expose_secret().as_ref()),
        &validation,
    )
    .map_err(|e| ApiError::InvalidAuthToken(e.into()));

    result
}

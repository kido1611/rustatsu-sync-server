use anyhow::Context;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};

use crate::util::AuthError;

#[derive(Clone)]
pub struct UserId(pub i64);

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

#[tracing::instrument(name = "Create JWT token", skip(user), fields(user.id))]
pub fn create_token(user: User) -> Result<String, AuthError> {
    // TODO: use config
    let secret = "my-secret-key".to_string();
    let iss = "http://localhost:8080/".to_string();
    let aud = "http://localhost:8080/resource".to_string();
    let now = Utc::now();
    let expire: chrono::TimeDelta = Duration::hours(24);
    let exp: usize = (now + expire).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claim = Claim {
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

#[tracing::instrument(name = "Decode JWT token", skip(jwt_token))]
pub fn decode_jwt(jwt_token: String) -> Result<TokenData<Claim>, AuthError> {
    // TODO: use confiig
    let secret = "my-secret-key".to_string();
    let iss = "http://localhost:8080/".to_string();
    let aud = "http://localhost:8080/resource".to_string();

    let mut validation = Validation::default();
    validation.set_issuer(&[iss]);
    validation.set_audience(&[aud]);

    let result = decode::<Claim>(
        &jwt_token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    )
    .map_err(|e| AuthError::UnexpectedError(e.into()));

    result
}

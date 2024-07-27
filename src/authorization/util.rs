use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};

use crate::util::AuthError;

#[derive(Clone)]
pub struct UserId(pub i64);

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Claim {
    pub user_id: i64,
    pub aud: String,
    pub iss: String,
    pub exp: usize,
    pub iat: usize,
}

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
    .map_err(|e| {
        dbg!(&e);
        AuthError::UnexpectedError(e.into())
    });

    dbg!(&jwt_token);

    result
}

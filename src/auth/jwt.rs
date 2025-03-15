use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode};
use secrecy::ExposeSecret;

use crate::{config::Jwt, error::Error};

use super::error::AuthError;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Claim {
    pub user_id: i64,
    aud: String,
    iss: String,
    exp: usize,
    iat: usize,
}

pub fn encode_jwt(user_id: i64, jwt: &Jwt) -> Result<String, Error> {
    let now = Utc::now();
    let expire: chrono::TimeDelta = Duration::hours(24);
    let exp: usize = (now + expire).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claim = Claim {
        user_id,
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
    .map_err(|e| Error::Auth(AuthError::JwtError(e)));

    result
}

pub fn decode_jwt(jwt_token: String, jwt: &Jwt) -> Result<TokenData<Claim>, Error> {
    let mut validation = Validation::default();
    validation.set_issuer(&[jwt.iss.expose_secret()]);
    validation.set_audience(&[jwt.aud.expose_secret()]);

    let result = decode::<Claim>(
        &jwt_token,
        &DecodingKey::from_secret(jwt.secret.expose_secret().as_ref()),
        &validation,
    )
    .map_err(|e| Error::Auth(AuthError::JwtError(e)));

    result
}

#[cfg(test)]
mod tests {
    use crate::config::Jwt;

    use super::{decode_jwt, encode_jwt};

    #[tokio::test]
    async fn can_encode_decode_jwt() {
        let jwt = Jwt {
            secret: "this is secret".into(),
            iss: "rustatsu".into(),
            aud: "rustatsu".into(),
        };

        let encoded_jwt_result = encode_jwt(10i64, &jwt);
        assert!(encoded_jwt_result.is_ok());

        let jwt_token = encoded_jwt_result.unwrap();

        let decoded_jwt_result = decode_jwt(jwt_token, &jwt);
        assert!(decoded_jwt_result.is_ok());

        let token_data = decoded_jwt_result.unwrap();
        assert_eq!(10, token_data.claims.user_id);
    }

    #[tokio::test]
    async fn error_when_jwt_is_invalid() {
        let jwt_encode = Jwt {
            secret: "this is secret encode".into(),
            iss: "rustatsu".into(),
            aud: "rustatsu".into(),
        };
        let jwt_decode = Jwt {
            secret: "this is secret decode".into(),
            iss: "rustatsu".into(),
            aud: "rustatsu".into(),
        };

        let encoded_jwt_result = encode_jwt(10i64, &jwt_encode);
        assert!(encoded_jwt_result.is_ok());

        let jwt_token = encoded_jwt_result.unwrap();

        let decoded_jwt_result = decode_jwt(jwt_token, &jwt_decode);
        assert!(decoded_jwt_result.is_err());
    }
}

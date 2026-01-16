use std::slice::from_ref;

use chrono::{Duration, Utc};
use core_domain::{
    security::{auth_token::AuthToken, model::AuthTokenClaim},
    shared::error::DomainError,
};
use serde::{Deserialize, Serialize};

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use tracing::{Level, instrument};

#[derive(Serialize, Deserialize)]
struct JwtClaim {
    user_id: i64,
    aud: String,
    iss: String,
    exp: usize,
    iat: usize,
}

fn to_jwt_claim(auth_claim: &AuthTokenClaim, iss: String, aud: String) -> JwtClaim {
    let now = Utc::now();
    let expire: chrono::TimeDelta = Duration::hours(24);
    let exp: usize = (now + expire).timestamp() as usize;
    let iat = now.timestamp() as usize;

    JwtClaim {
        user_id: auth_claim.user_id,
        aud,
        iss,
        exp,
        iat,
    }
}

pub struct JwtAuthToken {
    pub secret: String,
    pub iss: String,
    pub aud: String,
}

impl AuthToken for JwtAuthToken {
    #[instrument(name = "security::encode_auth_token", skip_all, fields(use = "jwt"), level = Level::DEBUG)]
    fn encode(
        &self,
        claim: &core_domain::security::model::AuthTokenClaim,
    ) -> Result<String, core_domain::shared::error::DomainError> {
        let claim = to_jwt_claim(claim, self.iss.clone(), self.aud.clone());

        encode(
            &Header::default(),
            &claim,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| DomainError::AuthTokenError(e.into()))
    }

    #[instrument(name = "security::decode_auth_token", skip_all, fields(use = "jwt"), level = Level::DEBUG)]
    fn decode(
        &self,
        token: &str,
    ) -> Result<core_domain::security::model::AuthTokenClaim, core_domain::shared::error::DomainError>
    {
        let mut validation = Validation::default();
        validation.set_issuer(from_ref(&self.iss));
        validation.set_audience(from_ref(&self.aud));

        let claim = decode::<JwtClaim>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &validation,
        )
        .map_err(|e| DomainError::AuthTokenError(e.into()))?;

        Ok(AuthTokenClaim {
            user_id: claim.claims.user_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use core_domain::security::{auth_token::AuthToken, model::AuthTokenClaim};

    use crate::security::jwt_auth_token::JwtAuthToken;

    #[tokio::test]
    async fn can_encode_decode_jwt() {
        let jwt_auth_token = JwtAuthToken {
            secret: "this is secret".into(),
            iss: "rustatsu".into(),
            aud: "rustatsu".into(),
        };

        let encoded_jwt_result = jwt_auth_token
            .encode(&AuthTokenClaim { user_id: 10i64 })
            .unwrap();

        let token_data = jwt_auth_token.decode(&encoded_jwt_result).unwrap();

        assert_eq!(10, token_data.user_id);
    }

    #[tokio::test]
    async fn error_when_jwt_is_invalid() {
        let jwt_auth_token_encode = JwtAuthToken {
            secret: "this is secret encode".into(),
            iss: "rustatsu".into(),
            aud: "rustatsu".into(),
        };

        let jwt_auth_token_decode = JwtAuthToken {
            secret: "this is secret decode".into(),
            iss: "rustatsu".into(),
            aud: "rustatsu".into(),
        };

        let encoded_jwt_result = jwt_auth_token_encode
            .encode(&AuthTokenClaim { user_id: 10i64 })
            .unwrap();

        let token_data_result = jwt_auth_token_decode.decode(&encoded_jwt_result);
        assert!(token_data_result.is_err());
    }
}

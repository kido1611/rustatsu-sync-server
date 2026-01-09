use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use core_domain::{security::token_hasher::TokenHasher, shared::error::DomainError};
use hmac::{Hmac, KeyInit, Mac};
use subtle::ConstantTimeEq;
use tracing::{Level, instrument};

type HmacSha256 = Hmac<sha2::Sha256>;

pub struct HmacSha256TokenHasher {
    secret: Vec<u8>,
}

impl HmacSha256TokenHasher {
    pub fn new(secret: String) -> Self {
        Self {
            secret: secret.as_bytes().to_vec(),
        }
    }
}

impl TokenHasher for HmacSha256TokenHasher {
    #[instrument(name = "security::token_hasher_hash", skip_all, fields(use = "hmac-sha256"), level = Level::DEBUG, err(level = Level::ERROR))]
    fn hash(&self, token: &str) -> Result<String, DomainError> {
        let mut mac = HmacSha256::new_from_slice(&self.secret)
            .map_err(|e| DomainError::HashTokenError(e.into()))?;
        mac.update(token.as_bytes());

        Ok(URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes()))
    }

    #[instrument(name = "security::token_hasher_verify", skip_all, fields(use = "hmac-sha256"), level = Level::DEBUG, err(level = Level::ERROR))]
    fn verify(&self, token: &str, hashed_token: &str) -> Result<bool, DomainError> {
        let computed_token = self.hash(token)?;

        Ok(computed_token
            .as_bytes()
            .ct_eq(hashed_token.as_bytes())
            .into())
    }
}

#[cfg(test)]
mod tests {
    use core_domain::security::token_hasher::TokenHasher;

    use crate::security::hmac_sha256_token_hasher::HmacSha256TokenHasher;

    #[test]
    fn should_be_success_when_hash_and_verify_token() {
        let token_hasher = HmacSha256TokenHasher::new("secret".to_string());

        let hashed_token = token_hasher.hash("this-is-token");
        assert!(hashed_token.is_ok());

        let verify_hash = token_hasher.verify("this-is-token", &hashed_token.unwrap());
        assert!(verify_hash.is_ok());
        assert!(verify_hash.unwrap());
    }

    #[test]
    fn should_be_false_when_verify_invalid_token() {
        let token_hasher = HmacSha256TokenHasher::new("secret".to_string());

        let hashed_token = token_hasher.hash("this-is-token");
        assert!(hashed_token.is_ok());

        let verify_hash = token_hasher.verify("this-is-invalid-token", &hashed_token.unwrap());
        assert!(verify_hash.is_ok());
        assert!(!verify_hash.unwrap());
    }

    #[test]
    fn should_be_false_when_hashed_token_is_invalid() {
        let token_hasher = HmacSha256TokenHasher::new("secret".to_string());

        let verify_hash = token_hasher.verify("this-is-token", "this-is-not-hmac-token");
        assert!(verify_hash.is_ok());
        assert!(!verify_hash.unwrap());
    }
}

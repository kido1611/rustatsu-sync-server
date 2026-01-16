use crate::{security::model::AuthTokenClaim, shared::error::DomainError};

pub trait AuthToken: Send + Sync {
    fn encode(&self, claim: &AuthTokenClaim) -> Result<String, DomainError>;
    fn decode(&self, token: &str) -> Result<AuthTokenClaim, DomainError>;
}

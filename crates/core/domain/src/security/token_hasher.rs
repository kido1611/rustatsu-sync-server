use crate::shared::error::DomainError;

pub trait TokenHasher: Send + Sync {
    fn hash(&self, token: &str) -> Result<String, DomainError>;
    fn verify(&self, token: &str, hashed_token: &str) -> Result<bool, DomainError>;
}

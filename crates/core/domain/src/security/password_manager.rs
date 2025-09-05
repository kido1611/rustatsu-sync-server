use crate::shared::error::DomainError;

pub trait PasswordManager: Send + Sync {
    fn hash_password(&self, password: &str) -> Result<String, DomainError>;
    fn verify(&self, password_hashed: &str, password: &str) -> Result<(), DomainError>;
}

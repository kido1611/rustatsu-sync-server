use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use core_domain::{security::password_manager::PasswordManager, shared::error::DomainError};
use tracing::{Level, instrument};

pub struct ArgonPasswordManager {}

impl PasswordManager for ArgonPasswordManager {
    #[instrument(name = "security::hash_password", skip_all, fields(use = "argon2id"), level = Level::DEBUG, err(level = Level::ERROR))]
    fn hash_password(&self, password: &str) -> Result<String, DomainError> {
        let password_hash = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None).unwrap(),
        )
        .hash_password(password.as_bytes())
        .map_err(|e| DomainError::PasswordManagerError(e.into()))?
        .to_string();

        Ok(password_hash)
    }

    #[instrument(name = "security::verify_password", skip_all, fields(use = "argon2id"), level = Level::DEBUG, err(level = Level::ERROR))]
    fn verify(&self, password_hashed: &str, password: &str) -> Result<(), DomainError> {
        let expected_password_hash = PasswordHash::new(password_hashed)
            .map_err(|e| DomainError::PasswordManagerError(e.into()))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &expected_password_hash)
            .map_err(|_| DomainError::PasswordNotMatch)
    }
}

#[cfg(test)]
mod tests {
    use core_domain::security::password_manager::PasswordManager;

    use crate::security::argon_password_manager::ArgonPasswordManager;

    #[test]
    fn should_be_success_to_hash_and_verify_password() {
        let argon_password_manager = ArgonPasswordManager {};
        let password_hash_result = argon_password_manager.hash_password("password");
        assert!(password_hash_result.is_ok());

        let password_hash = password_hash_result.unwrap();
        let verify_result = argon_password_manager.verify(&password_hash, "password");
        assert!(verify_result.is_ok());
    }

    #[test]
    fn should_be_error_when_verify_incorrect_password() {
        let argon_password_manager = ArgonPasswordManager {};
        let password_hash_result = argon_password_manager.hash_password("password");
        assert!(password_hash_result.is_ok());

        let password_hash = password_hash_result.unwrap();
        let verify_result = argon_password_manager.verify(&password_hash, "password-incorrect");
        assert!(verify_result.is_err());
    }

    #[test]
    fn should_be_error_when_password_hash_incorrect() {
        let argon_password_manager = ArgonPasswordManager {};
        let verify_result =
            argon_password_manager.verify("incorrect_password-hash", "password-incorrect");
        assert!(verify_result.is_err());
    }
}

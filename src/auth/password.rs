use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::{SaltString, rand_core::OsRng},
};
use secrecy::{ExposeSecret, SecretString};

use crate::error::Error;

use super::error::AuthError;

#[tracing::instrument(name = "compute password hash", skip_all)]
pub fn compute_password_hash(password: SecretString) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    )
    .hash_password(password.expose_secret().as_bytes(), &salt)
    .map_err(|e| Error::Auth(AuthError::PasswordError(e)))?
    .to_string();

    Ok(password_hash)
}

#[tracing::instrument(name = "verify password hash", skip_all)]
pub fn verify_password_hash(password_hashed: String, password: SecretString) -> Result<(), Error> {
    let expected_password_hash = PasswordHash::new(&password_hashed)
        .map_err(|e| Error::Auth(AuthError::PasswordError(e)))?;

    Argon2::default()
        .verify_password(password.expose_secret().as_bytes(), &expected_password_hash)
        .map_err(|e| Error::Auth(AuthError::PasswordError(e)))
}

#[cfg(test)]
mod tests {
    use super::{compute_password_hash, verify_password_hash};

    #[tokio::test]
    async fn can_compute_and_verify_password() {
        let hash_result = compute_password_hash("password".into());
        assert!(hash_result.is_ok());

        let password_hash = hash_result.unwrap();
        let verify_result = verify_password_hash(password_hash, "password".into());
        assert!(verify_result.is_ok());
    }

    #[tokio::test]
    async fn error_when_verify_incorrect_password() {
        let hash_result = compute_password_hash("password".into());
        assert!(hash_result.is_ok());

        let password_hash = hash_result.unwrap();
        let verify_result = verify_password_hash(password_hash, "password2".into());
        assert!(verify_result.is_err());
    }
}

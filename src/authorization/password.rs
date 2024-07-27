use anyhow::Context;
use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};
use secrecy::{ExposeSecret, Secret};

#[tracing::instrument(name = "calculate password hash", skip(password))]
pub fn compute_password_hash(password: Secret<String>) -> Result<Secret<String>, anyhow::Error> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    )
    .hash_password(password.expose_secret().as_bytes(), &salt)?
    .to_string();

    Ok(Secret::new(password_hash))
}

#[tracing::instrument(name = "verify password", skip_all)]
pub fn verify_password_hash(
    expected_password: Secret<String>,
    password: Secret<String>,
) -> Result<(), anyhow::Error> {
    let expected_password_hash = PasswordHash::new(expected_password.expose_secret())
        .context("Failed to parse bash in PHC string format.")?;

    Argon2::default()
        .verify_password(password.expose_secret().as_bytes(), &expected_password_hash)
        .context("Invalid password")
}

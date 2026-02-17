use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use core_domain::security::token_generator::TokenGenerator;
use rand::{Rng, rng};
use tracing::{Level, instrument};

pub struct SecureRandomTokenGenerator {}

impl TokenGenerator for SecureRandomTokenGenerator {
    #[instrument(name = "security::genereate_token", skip_all, fields(use = "rng"), level = Level::DEBUG)]
    fn generate(&self, length: usize) -> String {
        let mut bytes = vec![0u8; length];

        rng().fill_bytes(&mut bytes);

        URL_SAFE_NO_PAD.encode(bytes)
    }
}

#[cfg(test)]
mod tests {
    use core_domain::security::token_generator::TokenGenerator;

    use crate::security::secure_random_token_generator::SecureRandomTokenGenerator;

    #[test]
    fn can_generate_token() {
        let token_generator = SecureRandomTokenGenerator {};

        let token = token_generator.generate(20);

        assert!(!token.is_empty());
        // generated token length > 20
        assert!(token.len() > 20);
    }

    #[test]
    fn generated_token_should_be_unique() {
        let token_generator = SecureRandomTokenGenerator {};

        let token_1 = token_generator.generate(20);
        let token_2 = token_generator.generate(20);

        assert_ne!(token_1, token_2, "Generated token should be unique");
    }
}

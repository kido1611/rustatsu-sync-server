use std::sync::Arc;

use chrono::{Duration, Utc};
use core_domain::{
    notification::mailer::Mailer,
    security::{token_generator::TokenGenerator, token_hasher::TokenHasher},
    user::{model::UserPasswordReset, repository::UserRepository},
};
use tracing::{Level, instrument, warn};

use crate::{
    shared::error::ApplicationError, user::mail::forgot_password_mail::ForgotPasswordMail,
};

pub struct RequestResetPasswordUseCase {
    pub user_repository: Arc<dyn UserRepository>,
    pub token_generator: Arc<dyn TokenGenerator>,
    pub token_hasher: Arc<dyn TokenHasher>,
    pub mailer: Arc<dyn Mailer>,
}

impl RequestResetPasswordUseCase {
    #[instrument(name = "usecase::request_reset_user_password", skip_all, fields(email = %email) level = Level::DEBUG, err(level = Level::ERROR))]
    pub async fn execute(
        &self,
        email: &str,
        base_url: &str,
    ) -> Result<Option<String>, ApplicationError> {
        // get user by email
        let user = match self.user_repository.get_by_email(email).await? {
            Some(u) => u,
            None => return Ok(None),
        };

        let expires_at = user.password_reset_token_expires_at;
        let send_mail = match expires_at {
            Some(time) => {
                !(user.password_reset_token_hash.is_some() && time > Utc::now().timestamp())
            }
            None => true,
        };

        if send_mail {
            // generate token and expired date
            let token = self.token_generator.generate(32);
            let expires_at = Utc::now() + Duration::minutes(15);

            // hash token
            let hashed_token = self.token_hasher.hash(&token)?;

            // store token and expired date to database
            self.user_repository
                .update_password_reset_token(UserPasswordReset {
                    user_id: user.id,
                    token: hashed_token,
                    expires_at: expires_at.timestamp(),
                })
                .await?;

            let mail = ForgotPasswordMail {
                url: format!("{}/deeplink/reset-password?token={}", base_url, token),
            }
            .to_mail_envelope(user.email);

            // TODO: use queue / worker
            self.mailer.send(mail).await?;

            return Ok(Some(token));
        }

        warn!("already sending forgot-password mail");

        Ok(None)
    }
}

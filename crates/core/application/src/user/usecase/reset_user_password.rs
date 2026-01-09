use std::sync::Arc;

use core_domain::{
    security::{password_manager::PasswordManager, token_hasher::TokenHasher},
    user::{model::UserUpdatePassword, repository::UserRepository},
};
use secrecy::ExposeSecret;
use tracing::{Level, instrument};

use crate::{shared::error::ApplicationError, user::model::UserResetPasswordInput};

pub struct ResetUserPasswordUseCase {
    pub user_repository: Arc<dyn UserRepository>,
    pub password_manager: Arc<dyn PasswordManager>,
    pub token_hasher: Arc<dyn TokenHasher>,
}

impl ResetUserPasswordUseCase {
    #[instrument(name = "usecase::reset_user_password", skip_all, level = Level::DEBUG, err(level = Level::ERROR))]
    pub async fn execute(&self, data: UserResetPasswordInput) -> Result<(), ApplicationError> {
        // TODO: should be make issue to add email as POST request to reduce query usage
        let users = self
            .user_repository
            .list_by_active_password_reset_token()
            .await?;

        let user = users
            .iter()
            .find(|u| {
                u.password_reset_token_hash
                    .as_deref()
                    .map(|hash| {
                        self.token_hasher
                            .verify(data.token.expose_secret().trim(), hash.trim())
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
            })
            .ok_or(ApplicationError::ResourceNotFound(
                "user is missing".to_string(),
            ))?;

        // hash new password
        let hashed_password = self
            .password_manager
            .hash_password(data.password.expose_secret().trim())?;

        // update user password
        self.user_repository
            .update_user_password(UserUpdatePassword {
                user_id: user.id,
                password: hashed_password,
            })
            .await?;

        Ok(())
    }
}

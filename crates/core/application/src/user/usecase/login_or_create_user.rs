use std::sync::Arc;

use core_domain::{
    security::{auth_token::AuthToken, model::AuthTokenClaim, password_manager::PasswordManager},
    user::repository::UserRepository,
};
use secrecy::ExposeSecret;
use tracing::{Level, instrument};

use crate::{
    shared::error::ApplicationError,
    user::model::{UserDto, UserInput},
};

pub struct LoginOrCreateUserUseCase {
    pub user_repository: Arc<dyn UserRepository>,
    pub password_manager: Arc<dyn PasswordManager>,
    pub auth_token: Arc<dyn AuthToken>,
    pub allow_to_register: bool,
}

impl LoginOrCreateUserUseCase {
    #[instrument(name = "usecase::login_or_create_user_use_case", skip_all, fields(email = %data.email, allow_to_register = %self.allow_to_register), level = Level::DEBUG, err(level = Level::ERROR))]
    pub async fn execute(&self, data: UserInput) -> Result<(UserDto, String), ApplicationError> {
        // check user by email
        let user_option = self.user_repository.get_by_email(&data.email).await?;

        let user = if let Some(user) = user_option {
            // if user exist, check password
            self.password_manager
                .verify(&user.password, data.password.expose_secret())?;

            user
        } else {
            // user is missing, register user if allowed
            if !self.allow_to_register {
                return Err(ApplicationError::RegisterIsForbidden);
            }

            // create new user
            let hashed_password = self
                .password_manager
                .hash_password(data.password.expose_secret())?;

            self.user_repository
                .create(core_domain::user::model::UserCreate {
                    email: data.email,
                    password: hashed_password,
                    nickname: data.nickname,
                })
                .await?
        };

        let token = self
            .auth_token
            .encode(&AuthTokenClaim { user_id: user.id })?;

        Ok((user.into(), token))
    }
}

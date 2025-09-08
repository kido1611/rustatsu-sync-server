use std::sync::Arc;

use core_domain::{security::password_manager::PasswordManager, user::repository::UserRepository};
use secrecy::ExposeSecret;
use tracing::instrument;

use crate::{
    shared::error::ApplicationError,
    user::model::{UserDto, UserInput},
};

pub struct CheckOrCreateUserUsecase {
    pub user_repository: Arc<dyn UserRepository>,
    pub password_manager: Arc<dyn PasswordManager>,
    pub allow_to_register: bool,
}

impl CheckOrCreateUserUsecase {
    #[instrument(name = "usecase::check_or_create_user", skip_all, fields(email = %data.email, allow_to_register = %self.allow_to_register))]
    pub async fn execute(&self, data: UserInput) -> Result<UserDto, ApplicationError> {
        // check user by email
        let user_option = self.user_repository.get_by_email(&data.email).await?;

        // if user is exist
        if let Some(user) = user_option {
            // check password
            self.password_manager
                .verify(&user.password, data.password.expose_secret())?;

            return Ok(user.into());
        }

        // because user is None, check is allowed to register
        if !self.allow_to_register {
            return Err(ApplicationError::RegisterIsForbidden);
        }

        // create new user
        let hashed_password = self
            .password_manager
            .hash_password(data.password.expose_secret())?;
        let user = self
            .user_repository
            .create(core_domain::user::model::UserCreate {
                email: data.email,
                password: hashed_password,
                nickname: data.nickname,
            })
            .await?;

        Ok(user.into())
    }
}

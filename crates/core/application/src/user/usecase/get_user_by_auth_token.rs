use std::sync::Arc;

use core_domain::{security::auth_token::AuthToken, user::repository::UserRepository};
use tracing::{Level, instrument};

use crate::{shared::error::ApplicationError, user::model::UserDto};

pub struct GetUserByAuthTokenUseCase {
    pub user_repository: Arc<dyn UserRepository>,
    pub auth_token: Arc<dyn AuthToken>,
}

impl GetUserByAuthTokenUseCase {
    #[instrument(name = "usecase::get_user_by_auth_token", skip_all, level = Level::DEBUG, err(level = Level::ERROR))]
    pub async fn execute(&self, token: &str) -> Result<Option<UserDto>, ApplicationError> {
        let auth_claim_token = self.auth_token.decode(token)?;

        let user: Option<UserDto> = self
            .user_repository
            .get_by_id(auth_claim_token.user_id)
            .await?
            .map(|val| val.into());

        Ok(user)
    }
}

use std::sync::Arc;

use core_domain::user::repository::UserRepository;
use tracing::instrument;

use crate::{shared::error::ApplicationError, user::model::UserDto};

pub struct CheckUserByIdUsecase {
    pub user_repository: Arc<dyn UserRepository>,
}

impl CheckUserByIdUsecase {
    #[instrument(name = "usecase::check_user_by_id", skip_all, fields(user_id = %user_id))]
    pub async fn execute(&self, user_id: i64) -> Result<Option<UserDto>, ApplicationError> {
        let user: Option<UserDto> = self
            .user_repository
            .get_by_id(user_id)
            .await?
            .map(|val| val.into());

        Ok(user)
    }
}

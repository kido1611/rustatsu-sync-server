use std::sync::Arc;

use axum::{
    Extension,
    extract::{Json, State},
};
use shared::model::{request::UserFavouriteRequest, response::UserFavouriteResponse};

use crate::{error::Error, model::User, state::SharedAppState};

#[tracing::instrument(name = "request::list_user_favourite", skip_all)]
pub async fn index(
    Extension(user): Extension<Arc<User>>,
    State(app_state): State<SharedAppState>,
) -> Result<axum::Json<UserFavouriteResponse>, Error> {
    let user_favourite_resource = app_state
        .get_user_favourite_resource_usecase
        .execute(user.id)
        .await?;

    Ok(Json(user_favourite_resource.into()))
}

#[tracing::instrument(name = "request::update_user_favourite", skip_all)]
pub async fn store(
    Extension(user): Extension<Arc<User>>,
    State(app_state): State<SharedAppState>,
    Json(user_favourite): Json<UserFavouriteRequest>,
) -> Result<axum::Json<UserFavouriteResponse>, Error> {
    app_state
        .insert_user_favourite_resource_usecase
        .execute(user.id, user_favourite.into())
        .await?;

    let user_favourite_resource = app_state
        .get_user_favourite_resource_usecase
        .execute(user.id)
        .await?;

    Ok(Json(user_favourite_resource.into()))
}

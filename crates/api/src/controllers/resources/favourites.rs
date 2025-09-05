use std::sync::Arc;

use axum::{
    Extension,
    extract::{Json, State},
};

use crate::{
    db::user_favourites::{get_user_favourites, update_user_favourites},
    error::Error,
    model::{User, UserFavourite},
    state::SharedAppState,
};

#[tracing::instrument(name = "[GET] resource favourites", skip_all)]
pub async fn index(
    Extension(user): Extension<Arc<User>>,
    State(app_state): State<SharedAppState>,
) -> Result<axum::Json<UserFavourite>, Error> {
    let result = get_user_favourites(&app_state.pool, user.id).await?;
    Ok(Json(result))
}

#[tracing::instrument(name = "[POST] resource favourites", skip_all)]
pub async fn store(
    Extension(user): Extension<Arc<User>>,
    State(app_state): State<SharedAppState>,
    Json(user_favourite): Json<UserFavourite>,
) -> Result<axum::Json<UserFavourite>, Error> {
    update_user_favourites(&app_state.pool, user.id, user_favourite).await?;

    let result = get_user_favourites(&app_state.pool, user.id).await?;
    Ok(Json(result))
}

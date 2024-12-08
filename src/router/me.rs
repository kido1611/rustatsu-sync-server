use std::sync::Arc;

use anyhow::Context;
use axum::{extract::State, Extension, Json};

use crate::{
    authorization::{User, UserId},
    error::ApiError,
    startup::AppState,
};

#[tracing::instrument(name = "get me route", skip(app_state, user), fields(user_id=user.0))]
pub async fn get_me_route(
    State(app_state): State<Arc<AppState>>,
    Extension(user): Extension<UserId>,
) -> Result<Json<User>, ApiError> {
    let user = user
        .to_user(&app_state.pool)
        .await
        .context("User is missing")
        .map_err(ApiError::UnexpectedError)?;

    let user = match user {
        Some(user) => user,
        None => return Err(ApiError::UserMissing(anyhow::anyhow!("User not found"))),
    };

    Ok(Json(user))
}

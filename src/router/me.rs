use anyhow::Context;
use axum::{extract::State, Extension, Json};

use crate::{
    authorization::{User, UserId},
    startup::AppState,
    util::AuthError,
};

#[tracing::instrument(name = "Get user", skip(app_state))]
pub async fn get_user(
    State(app_state): State<AppState>,
    Extension(user): Extension<UserId>,
) -> Result<Json<User>, AuthError> {
    let user = user
        .to_user(&app_state.pool)
        .await
        .context("User is missing")
        .map_err(AuthError::UnexpectedError)?;

    let user = match user {
        Some(user) => user,
        None => return Err(AuthError::UserMissing(anyhow::anyhow!("User not found"))),
    };

    Ok(Json(user))
}

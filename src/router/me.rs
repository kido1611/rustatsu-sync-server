use anyhow::Context;
use axum::{extract::State, Extension, Json};
use sqlx::MySqlPool;

use crate::{
    authorization::{User, UserId},
    util::AuthError,
};

#[tracing::instrument(name = "Get user", skip(pool))]
pub async fn get_user(
    State(pool): State<MySqlPool>,
    Extension(user): Extension<UserId>,
) -> Result<Json<User>, AuthError> {
    let user = user
        .to_user(&pool)
        .await
        .context("User is missing")
        .map_err(AuthError::UnexpectedError)?;

    let user = match user {
        Some(user) => user,
        None => return Err(AuthError::UserMissing(anyhow::anyhow!("User not found"))),
    };

    Ok(Json(user))
}

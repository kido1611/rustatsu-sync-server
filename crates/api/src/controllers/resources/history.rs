use std::sync::Arc;

use axum::{Extension, Json, extract::State};

use crate::{
    db::user_history::{get_user_history, update_user_history},
    error::Error,
    model::{User, UserHistory},
    state::SharedAppState,
};

#[tracing::instrument(name = "[GET] resource history", skip_all)]
pub async fn index(
    Extension(user): Extension<Arc<User>>,
    State(app_state): State<SharedAppState>,
) -> Result<Json<UserHistory>, Error> {
    let result = get_user_history(&app_state.pool, user.id).await?;

    Ok(Json(result))
}

#[tracing::instrument(name = "[POST] resource history", skip_all)]
pub async fn store(
    Extension(user): Extension<Arc<User>>,
    State(app_state): State<SharedAppState>,
    axum::extract::Json(user_history): axum::extract::Json<UserHistory>,
) -> Result<Json<UserHistory>, Error> {
    update_user_history(&app_state.pool, user.id, user_history).await?;

    let result = get_user_history(&app_state.pool, user.id).await?;

    Ok(Json(result))
}

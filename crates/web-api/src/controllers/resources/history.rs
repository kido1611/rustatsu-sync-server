use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use shared::model::{request::UserHistoryRequest, response::UserHistoryResponse};

use crate::{error::Error, model::User, state::SharedAppState};

#[tracing::instrument(name = "request::list_user_history", skip_all)]
pub async fn index(
    Extension(user): Extension<Arc<User>>,
    State(app_state): State<SharedAppState>,
) -> Result<Json<UserHistoryResponse>, Error> {
    let history_resource = app_state
        .get_user_history_resource_usecase
        .execute(user.id)
        .await?;

    Ok(Json(history_resource.into()))
}

#[tracing::instrument(name = "request::update_user_history", skip_all)]
pub async fn store(
    Extension(user): Extension<Arc<User>>,
    State(app_state): State<SharedAppState>,
    axum::extract::Json(user_history): axum::extract::Json<UserHistoryRequest>,
) -> Result<Json<UserHistoryResponse>, Error> {
    app_state
        .insert_user_history_resource_usecase
        .execute(user.id, user_history.into())
        .await?;

    let history_resource = app_state
        .get_user_history_resource_usecase
        .execute(user.id)
        .await?;

    Ok(Json(history_resource.into()))
}

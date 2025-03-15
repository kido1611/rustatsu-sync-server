use std::sync::Arc;

use axum::{Extension, Json};

use crate::{error::Error, model::User};

#[tracing::instrument(name = "[GET] me", skip_all)]
pub async fn index(Extension(user): Extension<Arc<User>>) -> Result<Json<Arc<User>>, Error> {
    Ok(Json(user))
}

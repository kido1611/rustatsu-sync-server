use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    db::manga::{get_manga_by_id, get_manga_with_pagination},
    error::Error,
    model::Manga,
    state::SharedAppState,
};
use serde_aux::field_attributes::deserialize_option_number_from_string;

#[tracing::instrument(name = "[GET] manga", skip_all)]
pub async fn index(
    State(app_state): State<SharedAppState>,
    Query(parameters): Query<Parameters>,
) -> Result<Json<Vec<Manga>>, Error> {
    let limit = parameters.limit.unwrap_or(20);
    let skip = parameters.offset.unwrap_or(0) * limit;

    let result = get_manga_with_pagination(&app_state.pool, limit, skip).await?;

    Ok(Json(result))
}

#[tracing::instrument(name = "[GET] manga/{id}", skip_all, fields(path.id))]
pub async fn show(
    State(app_state): State<SharedAppState>,
    Path(path): Path<UrlPath>,
) -> Result<Json<Manga>, Error> {
    let result = get_manga_by_id(&app_state.pool, path.id).await?;

    Ok(Json(result))
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Parameters {
    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    offset: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    limit: Option<i64>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct UrlPath {
    id: i64,
}

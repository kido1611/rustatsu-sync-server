use axum::{
    Json,
    extract::{Path, Query, State},
};
use validator::Validate;

use crate::{
    db::manga::{get_manga_by_id, get_manga_with_pagination},
    error::Error,
    model::Manga,
    state::SharedAppState,
};
use serde_aux::field_attributes::deserialize_option_number_from_string;

#[tracing::instrument(name = "[GET] manga", skip_all, fields(parameters))]
pub async fn index(
    State(app_state): State<SharedAppState>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<Manga>>, Error> {
    pagination.validate().map_err(Error::Validation)?;

    let limit = pagination.limit.unwrap_or(20);
    let skip = pagination.offset.unwrap_or(0) * limit;

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

#[derive(serde::Deserialize, serde::Serialize, Debug, Validate)]
pub struct Pagination {
    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    #[validate(range(min = 0))]
    offset: Option<i64>,

    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    #[validate(range(min = 0))]
    limit: Option<i64>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct UrlPath {
    id: i64,
}

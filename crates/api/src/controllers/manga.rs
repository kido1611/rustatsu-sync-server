use axum::{
    Json,
    extract::{Path, Query, State},
};
use core_application::manga::model::ListMangaQuery;
use shared::model::response::{MangaResponse, list_manga_resource_to_list_manga_response};
use validator::Validate;

use crate::{error::Error, state::SharedAppState};
use serde_aux::field_attributes::deserialize_option_number_from_string;

#[tracing::instrument(name = "request::list_manga", skip_all, fields(limit = ?pagination.limit, offset = ?pagination.offset))]
pub async fn index(
    State(app_state): State<SharedAppState>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<MangaResponse>>, Error> {
    pagination.validate().map_err(Error::Validation)?;

    let list_manga_resource = app_state
        .list_manga_usecase
        .execute(ListMangaQuery {
            limit: pagination.limit.unwrap_or(20),
            offset: pagination.offset.unwrap_or(0),
        })
        .await?;
    let list_manga_response = list_manga_resource_to_list_manga_response(list_manga_resource);

    Ok(Json(list_manga_response))
}

#[tracing::instrument(name = "request::get_manga_by_id", skip_all, fields(manga_id = %path.id))]
pub async fn show(
    State(app_state): State<SharedAppState>,
    Path(path): Path<UrlPath>,
) -> Result<Json<MangaResponse>, Error> {
    let manga: MangaResponse = app_state
        .get_manga_by_id_usecase
        .execute(path.id)
        .await?
        .ok_or(Error::ResourceNotFound(format!("manga {}", path.id)))?
        .into();

    Ok(Json(manga))
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

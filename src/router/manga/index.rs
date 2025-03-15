use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use serde_aux::field_attributes::deserialize_option_number_from_string;
use sqlx::MySqlPool;

use crate::error::Error;
use crate::model::{Manga, MangaEntity, TagEntity, transform_manga_entity_into_manga};
use crate::startup::AppState;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Parameters {
    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    offset: Option<u16>,
    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    limit: Option<u16>,
}

#[tracing::instrument(name = "get manga route", skip(app_state))]
pub async fn get_manga_route(
    State(app_state): State<Arc<AppState>>,
    Query(parameters): Query<Parameters>,
) -> Result<Json<Vec<Manga>>, Error> {
    let manga = get_manga_list(&app_state.pool, parameters)
        .await
        .map_err(|e| Error::UnexpectedError(e.into()))?;
    let manga_id: Vec<i64> = manga.iter().map(|m| m.id).collect();
    let tags = get_manga_tags_by_manga_id(&app_state.pool, manga_id)
        .await
        .map_err(|e| Error::UnexpectedError(e.into()))?;

    Ok(Json(transform_manga_entity_into_manga(manga, &tags)))
}

#[tracing::instrument(name = "get manga list", skip(pool))]
async fn get_manga_list(
    pool: &MySqlPool,
    parameters: Parameters,
) -> Result<Vec<MangaEntity>, sqlx::Error> {
    let limit = parameters.limit.unwrap_or(20);
    let skip = parameters.offset.unwrap_or(0) * limit;

    sqlx::query_as!(
        MangaEntity,
        r#"
               SELECT id, title, alt_title,
               url, public_url, rating, is_nsfw, cover_url, large_cover_url,
               state, author, source
               FROM manga
               ORDER BY id
               LIMIT ?
               OFFSET ?
               "#,
        limit,
        skip
    )
    .fetch_all(pool)
    .await
}

#[tracing::instrument(name = "get manga tags by manga id", skip(pool, manga_id))]
pub async fn get_manga_tags_by_manga_id(
    pool: &MySqlPool,
    manga_id: Vec<i64>,
) -> Result<Vec<TagEntity>, sqlx::Error> {
    if manga_id.is_empty() {
        return Ok(Vec::new());
    }

    let params = manga_id
        .iter()
        .map(|_| "?")
        .collect::<Vec<&str>>()
        .join(", ");
    let tag_sql = format!(
        "SELECT manga_tags.manga_id, tags.id, tags.title, tags.key, tags.source
                FROM tags
                JOIN manga_tags ON manga_tags.tag_id = tags.id
                WHERE manga_tags.manga_id IN ({})",
        params
    );
    let mut tag_rows = sqlx::query_as(&tag_sql);
    for mng in manga_id {
        tag_rows = tag_rows.bind(mng);
    }
    let tags: Result<Vec<TagEntity>, sqlx::Error> = tag_rows.fetch_all(pool).await;

    tags
}

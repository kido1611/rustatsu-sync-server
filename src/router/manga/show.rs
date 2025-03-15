use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use sqlx::MySqlPool;

use super::index::get_manga_tags_by_manga_id;
use crate::{
    error::Error,
    model::{Manga, MangaEntity},
    startup::AppState,
};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct UrlPath {
    id: i64,
}

#[tracing::instrument(name = "get manga id route", skip(app_state))]
pub async fn get_manga_id_route(
    State(app_state): State<Arc<AppState>>,
    Path(path): Path<UrlPath>,
) -> Result<Json<Manga>, Error> {
    let manga = get_manga(&app_state.pool, path.id)
        .await
        .map_err(|e| Error::UnexpectedError(e.into()))?;

    let manga = match manga {
        Some(m) => m,
        None => return Err(Error::Missing(anyhow::anyhow!("Manga is missing"))),
    };

    let manga_id = Vec::from([manga.id]);

    let tags = get_manga_tags_by_manga_id(&app_state.pool, manga_id)
        .await
        .map_err(|e| Error::UnexpectedError(e.into()))?;

    Ok(Json(Manga::from_entity(manga, &tags)))
}

#[tracing::instrument(name = "get manga by id", skip(pool))]
async fn get_manga(pool: &MySqlPool, id: i64) -> Result<Option<MangaEntity>, sqlx::Error> {
    sqlx::query_as!(
        MangaEntity,
        r#"
                   SELECT id, title, alt_title,
                   url, public_url, rating, is_nsfw, cover_url, large_cover_url,
                   state, author, source
                   FROM manga
                   WHERE id = ?
                   LIMIT 1
                   "#,
        id
    )
    .fetch_optional(pool)
    .await
}

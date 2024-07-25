use axum::{
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::MySqlPool;

use super::index::{get_manga_tags_by_manga_id, Manga, MangaEntity};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct UrlPath {
    id: i64,
}

#[tracing::instrument(name = "Get manga by id", skip(pool))]
pub async fn get_manga_by_id(
    State(pool): State<MySqlPool>,
    Path(path): Path<UrlPath>,
) -> Result<Json<Manga>, StatusCode> {
    let manga = sqlx::query_as!(
        MangaEntity,
        r#"
               SELECT id, title, alt_title,
               url, public_url, rating, is_nsfw, cover_url, large_cover_url,
               state, author, source
               FROM manga
               WHERE id = ?
               LIMIT 1
               "#,
        path.id
    )
    .fetch_optional(&pool)
    .await
    .unwrap();

    if manga.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    let manga = manga.unwrap();

    let manga_id = Vec::from([manga.id]);

    let tags = get_manga_tags_by_manga_id(&pool, manga_id).await.unwrap();

    Ok(Json(Manga::from_entity(manga, &tags)))
}

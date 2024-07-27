use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::MySqlPool;

use super::index::{get_manga_tags_by_manga_id, Manga, MangaEntity};
use crate::util::MangaError;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct UrlPath {
    id: i64,
}

#[tracing::instrument(name = "Get manga by id", skip(pool))]
pub async fn get_manga_by_id(
    State(pool): State<MySqlPool>,
    Path(path): Path<UrlPath>,
) -> Result<Json<Manga>, MangaError> {
    let manga = get_manga(&pool, path.id)
        .await
        .map_err(|e| MangaError::UnexpectedError(e.into()))?;

    let manga = match manga {
        Some(m) => m,
        None => return Err(MangaError::Missing(anyhow::anyhow!("Manga is missing"))),
    };

    let manga_id = Vec::from([manga.id]);

    let tags = get_manga_tags_by_manga_id(&pool, manga_id)
        .await
        .map_err(|e| MangaError::UnexpectedError(e.into()))?;

    Ok(Json(Manga::from_entity(manga, &tags)))
}

#[tracing::instrument(name = "Find manga by id", skip(pool))]
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

// #[derive(thiserror::Error, ErrorStatus)]
// pub enum MangaError {
//     #[error("Manga is missing")]
//     #[status(StatusCode::NOT_FOUND)]
//     Missing(#[source] anyhow::Error),

//     #[error("Something went wrong")]
//     #[status(StatusCode::INTERNAL_SERVER_ERROR)]
//     UnexpectedError(#[source] anyhow::Error),
// }

// impl std::fmt::Debug for MangaError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         error_chain_fmt(self, f)
//     }
// }

// pub fn error_chain_fmt(
//     e: &impl std::error::Error,
//     f: &mut std::fmt::Formatter<'_>,
// ) -> std::fmt::Result {
//     writeln!(f, "{}\n", e)?;
//     let mut current = e.source();
//     while let Some(cause) = current {
//         writeln!(f, "Caused by:\n\t{}", cause)?;
//         current = cause.source();
//     }

//     Ok(())
// }

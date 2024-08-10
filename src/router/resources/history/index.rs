use anyhow::Context;
use axum::{extract::State, Extension, Json};
use sqlx::MySqlPool;

use crate::{
    authorization::{User, UserId},
    router::manga::{get_manga_tags_by_manga_id, Manga, Tag},
    startup::AppState,
    util::MangaError,
};

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct HistoryPackage {
    pub history: Vec<History>,
    pub timestamp: i64,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct History {
    pub manga_id: i64,
    pub manga: Manga,
    pub created_at: i64,
    pub updated_at: i64,
    pub chapter_id: i64,
    pub page: i16,
    pub scroll: f64,
    pub percent: f64,
    pub chapters: i32,
    pub deleted_at: i64,
}

impl PartialEq for History {
    fn eq(&self, other: &Self) -> bool {
        if self.manga_id != other.manga_id {
            return false;
        }

        if self.created_at != other.created_at {
            return false;
        }

        if self.updated_at != other.updated_at {
            return false;
        }

        if self.chapter_id != other.chapter_id {
            return false;
        }

        if self.page != other.page {
            return false;
        }

        if self.scroll != other.scroll {
            return false;
        }

        if self.percent != other.percent {
            return false;
        }

        if self.chapters != other.chapters {
            return false;
        }

        if self.deleted_at != other.deleted_at {
            return false;
        }

        true
    }
}

impl Eq for History {}

#[tracing::instrument(
    name = "get history route",
    skip(app_state, user),
    fields(user_id = user.0)
)]
pub async fn get_history_route(
    State(app_state): State<AppState>,
    Extension(user): Extension<UserId>,
) -> Result<Json<HistoryPackage>, MangaError> {
    let user = user
        .to_user(&app_state.pool)
        .await
        .context("User is missing")
        .map_err(MangaError::UnexpectedError)?;

    let user = match user {
        Some(user) => user,
        None => {
            return Err(MangaError::InvalidCredential(anyhow::anyhow!(
                "User not found"
            )))
        }
    };

    let history_package = get_user_history_package(&app_state.pool, &user).await?;

    Ok(Json(history_package))
}

#[tracing::instrument(
    name = "get user history package",
    skip(pool, user),
    fields(user_id = user.id)
)]
pub async fn get_user_history_package(
    pool: &MySqlPool,
    user: &User,
) -> Result<HistoryPackage, MangaError> {
    let histories = get_user_history_manga(pool, user)
        .await
        .context("Error fetching history")
        .map_err(MangaError::UnexpectedError)?;

    let user_history_time = get_user_last_history_sync_time(pool, user)
        .await
        .context("Error fething user history sync time")
        .map_err(MangaError::UnexpectedError)?;

    let history_package = HistoryPackage {
        history: histories,
        timestamp: user_history_time,
    };

    Ok(history_package)
}

#[tracing::instrument(
    name = "get user history synchronize time",
    skip(pool, user),
    fields(user_id=user.id)
)]
async fn get_user_last_history_sync_time(
    pool: &MySqlPool,
    user: &User,
) -> Result<i64, sqlx::Error> {
    let user_history = sqlx::query!(
        r#"
        SELECT history_sync_timestamp
        FROM users
        WHERE id = ?
        "#,
        user.id
    )
    .fetch_one(pool)
    .await?;

    Ok(user_history.history_sync_timestamp.unwrap_or(0))
}

#[tracing::instrument(
    name = "get user history manga",
    skip(pool, user),
    fields(user_id=user.id)
)]
async fn get_user_history_manga(
    pool: &MySqlPool,
    user: &User,
) -> Result<Vec<History>, sqlx::Error> {
    let history_raw = sqlx::query!(
        r#"
            SELECT *
            FROM history
            INNER JOIN manga on history.manga_id = manga.id
            WHERE user_id = ?
        "#,
        user.id
    )
    .fetch_all(pool)
    .await?;

    let manga_ids = history_raw.iter().map(|h| h.manga_id).collect::<Vec<i64>>();

    let tags = get_manga_tags_by_manga_id(pool, manga_ids).await?;

    let histories = history_raw
        .iter()
        .map(|h| {
            let manga_tags = tags
                .iter()
                .filter(|t| t.manga_id == h.manga_id)
                .map(|t| Tag {
                    tag_id: t.id,
                    title: t.title.clone(),
                    key: t.key.clone(),
                    source: t.source.clone(),
                })
                .collect::<Vec<Tag>>();

            let manga = Manga {
                manga_id: h.manga_id,
                title: h.title.clone(),
                alt_title: h.alt_title.clone(),
                url: h.url.clone(),
                public_url: h.public_url.clone(),
                cover_url: h.cover_url.clone(),
                large_cover_url: h.large_cover_url.clone(),
                rating: h.rating,
                nsfw: h.is_nsfw,
                state: h.state.clone(),
                author: h.author.clone(),
                source: h.source.clone(),
                tags: manga_tags,
            };

            History {
                manga_id: h.manga_id,
                manga,
                created_at: h.created_at,
                chapters: h.chapters,
                updated_at: h.updated_at,
                chapter_id: h.chapter_id,
                page: h.page,
                scroll: h.scroll,
                percent: h.percent,
                deleted_at: h.deleted_at,
            }
        })
        .collect::<Vec<History>>();

    Ok(histories)
}

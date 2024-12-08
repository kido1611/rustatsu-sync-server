use std::sync::Arc;

use anyhow::Context;
use axum::{extract::State, Extension, Json};
use sqlx::MySqlPool;

use crate::{
    authorization::{User, UserId},
    error::ApiError,
    model::{Category, Favourite, FavouritePackage, Manga, Tag},
    router::manga::get_manga_tags_by_manga_id,
    startup::AppState,
};

#[tracing::instrument(
    name = "get favourites route",
    skip(app_state, user),
    fields(user_id = user.0)
)]
pub async fn get_favourites_route(
    State(app_state): State<Arc<AppState>>,
    Extension(user): Extension<UserId>,
) -> Result<Json<FavouritePackage>, ApiError> {
    let user = user
        .to_user(&app_state.pool)
        .await
        .context("User is missing")
        .map_err(ApiError::UnexpectedError)?;

    let user = match user {
        Some(user) => user,
        None => {
            return Err(ApiError::InvalidCredential(anyhow::anyhow!(
                "User not found"
            )))
        }
    };

    let favourites_package = get_user_favourites_package(&app_state.pool, &user).await?;

    Ok(Json(favourites_package))
}

#[tracing::instrument(
    name = "get user favourite package",
    skip(pool, user),
    fields(user_id = user.id)
)]
pub async fn get_user_favourites_package(
    pool: &MySqlPool,
    user: &User,
) -> Result<FavouritePackage, ApiError> {
    let categories = get_user_categories(pool, user)
        .await
        .context("Error fetching categories")
        .map_err(ApiError::UnexpectedError)?;

    let favourites = get_user_favourite_manga(pool, user)
        .await
        .map_err(ApiError::UnexpectedError)?;

    let favourite_time = get_user_last_favourite_sync_time(pool, user)
        .await
        .context("Failed fetching user last favourite sync time")
        .map_err(ApiError::UnexpectedError)?;

    let favourite_package = FavouritePackage {
        favourite_categories: categories,
        favourites,
        timestamp: favourite_time,
    };

    Ok(favourite_package)
}

#[tracing::instrument(
    name = "get user favourite synchronize time",
    skip(pool, user),
    fields(user_id=user.id)
)]
async fn get_user_last_favourite_sync_time(
    pool: &MySqlPool,
    user: &User,
) -> Result<i64, sqlx::Error> {
    let user = sqlx::query!(
        r#"
        SELECT favourites_sync_timestamp FROM users
        WHERE id = ?"#,
        &user.id
    )
    .fetch_one(pool)
    .await?;

    let time = user.favourites_sync_timestamp.unwrap_or(0);

    Ok(time)
}

#[tracing::instrument(
    name = "get user categories",
    skip(pool, user),
    fields(user_id=user.id)
)]
async fn get_user_categories(pool: &MySqlPool, user: &User) -> Result<Vec<Category>, sqlx::Error> {
    sqlx::query_as!(
        Category,
        r#"
            SELECT  id as category_id, created_at, sort_key,
                    title, `order`, track,
                    show_in_lib, deleted_at
            FROM categories
            WHERE user_id = ?
            LIMIT 10
        "#,
        user.id
    )
    .fetch_all(pool)
    .await
}

#[tracing::instrument(
    name = "get user favourite manga",
    skip(pool, user),
    fields(user_id=user.id)
)]
async fn get_user_favourite_manga(
    pool: &MySqlPool,
    user: &User,
) -> Result<Vec<Favourite>, anyhow::Error> {
    let favourites_with_manga = sqlx::query!(
        r#"
            SELECT *
            FROM favourites
            INNER JOIN manga on manga.id = favourites.manga_id
            WHERE user_id = ?
        "#,
        user.id
    )
    .fetch_all(pool)
    .await?;

    let manga_ids = favourites_with_manga
        .iter()
        .map(|f| f.manga_id)
        .collect::<Vec<i64>>();

    let tags = get_manga_tags_by_manga_id(pool, manga_ids).await?;

    let favourite = favourites_with_manga
        .iter()
        .map(|f| {
            let manga_tags = tags
                .iter()
                .filter(|t| t.manga_id == f.manga_id)
                .map(|t| Tag {
                    tag_id: t.id,
                    title: t.title.to_owned(),
                    key: t.key.to_owned(),
                    source: t.source.to_owned(),
                })
                .collect::<Vec<Tag>>();

            let manga = Manga {
                manga_id: f.manga_id,
                title: f.title.to_owned(),
                alt_title: f.alt_title.to_owned(),
                url: f.url.to_owned(),
                public_url: f.public_url.to_owned(),
                cover_url: f.cover_url.to_owned(),
                large_cover_url: f.large_cover_url.to_owned(),
                rating: f.rating,
                nsfw: f.is_nsfw,
                state: f.state.to_owned(),
                author: f.author.to_owned(),
                source: f.source.to_owned(),
                tags: manga_tags,
            };

            Favourite {
                manga_id: f.manga_id,
                manga,
                category_id: f.category_id,
                sort_key: f.sort_key,
                created_at: f.created_at,
                deleted_at: f.deleted_at,
            }
        })
        .collect::<Vec<Favourite>>();

    Ok(favourite)
}

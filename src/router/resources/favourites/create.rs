use anyhow::Context;
use axum::{extract::State, Extension, Json};
use sqlx::{Executor, MySql, MySqlPool, Transaction};

use super::{
    get_favourites_package_by_user,
    index::{Category, Favourite, FavouritesPackage},
};
use crate::{
    authorization::{User, UserId},
    router::manga::{Manga, Tag},
    startup::AppState,
    util::MangaError,
};

#[tracing::instrument(
    name = "Update Favourites",
    skip(app_state, user, favourites_package),
    fields(user_id=user.0)
)]
pub async fn post_favourites(
    State(app_state): State<AppState>,
    Extension(user): Extension<UserId>,
    axum::extract::Json(favourites_package): axum::extract::Json<FavouritesPackage>,
) -> Result<Json<FavouritesPackage>, MangaError> {
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

    let mut transaction = app_state
        .pool
        .begin()
        .await
        .context("Failed when creating database transaction")
        .map_err(MangaError::UnexpectedError)?;

    upsert_categories(
        &mut transaction,
        &user,
        &favourites_package.favourite_categories,
    )
    .await
    .context("Failed when updating categories")
    .map_err(MangaError::UnexpectedError)?;

    upsert_favourites(&mut transaction, &user, &favourites_package.favourites)
        .await
        .context("Failed when updating favourites")
        .map_err(MangaError::UnexpectedError)?;

    transaction
        .commit()
        .await
        .context("Failed when commiting transaction")
        .map_err(MangaError::UnexpectedError)?;

    let latest_favourites_package = get_favourites_package_by_user(&app_state.pool, &user).await?;

    update_user_favourite_synchonize_time(&app_state.pool, &user)
        .await
        .context("Failed when updating user favourite timestamp")
        .map_err(MangaError::UnexpectedError)?;

    if latest_favourites_package == favourites_package {
        return Err(MangaError::ContentEqual(anyhow::anyhow!("Content Equal")));
    }

    Ok(Json(latest_favourites_package))
}

async fn update_user_favourite_synchonize_time(
    pool: &MySqlPool,
    user: &User,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now();

    sqlx::query!(
        r#"
        UPDATE users
            set favourites_sync_timestamp = ?
        WHERE id = ?"#,
        now.timestamp(),
        user.id
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[tracing::instrument(
    name = "Upsert Categories",
    skip(transaction, user, categories),
    fields(user_id=user.id),
)]
async fn upsert_categories(
    transaction: &mut Transaction<'_, MySql>,
    user: &User,
    categories: &Vec<Category>,
) -> Result<(), sqlx::Error> {
    for category in categories {
        let query = sqlx::query!(
            r#"
                    INSERT INTO
                        categories (id, created_at, sort_key, title, `order`, user_id, track, show_in_lib, deleted_at)
                    VALUES
                        (?, ?, ?, ?, ?, ?, ?, ?, ?)
                    ON DUPLICATE KEY UPDATE
                        created_at = ?,
                        sort_key = ?,
                        title = ?,
                        `order` = ?,
                        user_id = ?,
                        track = ?,
                        show_in_lib = ?,
                        deleted_at = ?
                "#,
            category.category_id,
            category.created_at,
            category.sort_key,
            category.title,
            category.order,
            user.id,
            category.track,
            category.show_in_lib,
            category.deleted_at,
            category.created_at,
            category.sort_key,
            category.title,
            category.order,
            user.id,
            category.track,
            category.show_in_lib,
            category.deleted_at
        );

        transaction.execute(query).await?;
    }

    Ok(())
}

#[tracing::instrument(
    name = "Upsert favourite",
    skip(transaction, user, favourites),
    fields(user_id=user.id)
)]
async fn upsert_favourites(
    transaction: &mut Transaction<'_, MySql>,
    user: &User,
    favourites: &Vec<Favourite>,
) -> Result<(), sqlx::Error> {
    for favourite in favourites {
        let query = sqlx::query!(
            r#"
            INSERT INTO favourites
                (manga_id, category_id, sort_key, created_at, deleted_at, user_id)
            VALUES
                (?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
                sort_key = ?,
                created_at = ?,
                deleted_at = ?
            "#,
            favourite.manga_id,
            favourite.category_id,
            favourite.sort_key,
            favourite.created_at,
            favourite.deleted_at,
            user.id,
            favourite.sort_key,
            favourite.created_at,
            favourite.deleted_at,
        );
        transaction.execute(query).await?;

        upsert_manga(transaction, &favourite.manga).await?;
    }

    Ok(())
}

pub async fn upsert_manga(
    transaction: &mut Transaction<'_, MySql>,
    manga: &Manga,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"
            INSERT INTO manga
                (id, title, alt_title,
                url, public_url,
                rating, is_nsfw,
                cover_url, large_cover_url,
                state, author, source)
            VALUES
                (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
                title = ?,
                alt_title = ?,
                url = ?,
                public_url = ?,
                rating = ?,
                is_nsfw = ?,
                cover_url = ?,
                large_cover_url = ?,
                state = ?,
                author = ?,
                source = ?
            "#,
        manga.manga_id,
        manga.title.chars().take(84).collect::<String>(),
        manga
            .alt_title
            .clone()
            .map(|d| d.chars().take(84).collect::<String>()),
        manga.url.chars().take(255).collect::<String>(),
        manga.public_url.chars().take(255).collect::<String>(),
        manga.rating,
        manga.nsfw,
        manga.cover_url.chars().take(255).collect::<String>(),
        manga
            .large_cover_url
            .clone()
            .map(|d| d.chars().take(84).collect::<String>()),
        manga
            .state
            .clone()
            .map(|d| d.chars().take(84).collect::<String>()),
        manga
            .author
            .clone()
            .map(|d| d.chars().take(84).collect::<String>()),
        manga.source.chars().take(32).collect::<String>(),
        manga.title.chars().take(84).collect::<String>(),
        manga
            .alt_title
            .clone()
            .map(|d| d.chars().take(84).collect::<String>()),
        manga.url.chars().take(255).collect::<String>(),
        manga.public_url.chars().take(255).collect::<String>(),
        manga.rating,
        manga.nsfw,
        manga.cover_url.chars().take(255).collect::<String>(),
        manga
            .large_cover_url
            .clone()
            .map(|d| d.chars().take(84).collect::<String>()),
        manga
            .state
            .clone()
            .map(|d| d.chars().take(84).collect::<String>()),
        manga
            .author
            .clone()
            .map(|d| d.chars().take(84).collect::<String>()),
        manga.source.chars().take(32).collect::<String>()
    );
    transaction.execute(query).await?;

    upsert_tags(transaction, &manga, &manga.tags).await?;

    Ok(())
}

async fn upsert_tags(
    transaction: &mut Transaction<'_, MySql>,
    manga: &Manga,
    tags: &Vec<Tag>,
) -> Result<(), sqlx::Error> {
    for tag in tags {
        let query = sqlx::query!(
            r#"
                INSERT INTO tags
                    (id, title, `key`, source)
                VALUES
                    (?, ?, ?, ?)
                ON DUPLICATE KEY UPDATE
                    title = ?,
                    `key` = ?,
                    source = ?
            "#,
            tag.tag_id,
            tag.title.chars().take(64).collect::<String>(),
            tag.key.chars().take(120).collect::<String>(),
            tag.source.chars().take(32).collect::<String>(),
            tag.title.chars().take(64).collect::<String>(),
            tag.key.chars().take(120).collect::<String>(),
            tag.source.chars().take(32).collect::<String>(),
        );

        transaction.execute(query).await?;

        let query_manga_tag = sqlx::query!(
            r#"
            INSERT IGNORE INTO manga_tags
                (manga_id, tag_id)
            VALUES
                (?, ?)
            "#,
            manga.manga_id,
            tag.tag_id,
        );
        transaction.execute(query_manga_tag).await?;
    }
    Ok(())
}

use std::{fs::OpenOptions, sync::Arc};

use anyhow::Context;
use axum::{Extension, Json, extract::State};
use sqlx::{Executor, MySql, MySqlPool, Transaction};

use super::index::get_user_favourites_package;
use crate::{
    authorization::{User, UserId},
    error::Error,
    model::{Category, Favourite, FavouritePackage, Manga, Tag},
    startup::AppState,
};

#[tracing::instrument(
    name = "post favourite route",
    skip(app_state, user, favourites_package),
    fields(user_id=user.0)
)]
pub async fn post_favourites_route(
    State(app_state): State<Arc<AppState>>,
    Extension(user): Extension<UserId>,
    axum::extract::Json(favourites_package): axum::extract::Json<FavouritePackage>,
) -> Result<Json<FavouritePackage>, Error> {
    let user = user
        .to_user(&app_state.pool)
        .await
        .context("User is missing")
        .map_err(Error::UnexpectedError)?;

    let user = match user {
        Some(user) => user,
        None => return Err(Error::InvalidCredential(anyhow::anyhow!("User not found"))),
    };

    println!("favourite data: {}", &favourites_package.favourites.len());

    // write_to_file(&favourites_package).await?;

    let mut transaction = app_state
        .pool
        .begin()
        .await
        .context("Failed when creating database transaction")
        .map_err(Error::UnexpectedError)?;

    upsert_user_categories(
        &mut transaction,
        &user,
        &favourites_package.favourite_categories,
    )
    .await
    .context("Failed when updating categories")
    .map_err(Error::UnexpectedError)?;

    upsert_user_favourite_manga(&mut transaction, &user, &favourites_package.favourites)
        .await
        .context("Failed when updating favourites")
        .map_err(Error::UnexpectedError)?;

    transaction
        .commit()
        .await
        .context("Failed when commiting transaction")
        .map_err(Error::UnexpectedError)?;

    let latest_favourites_package = get_user_favourites_package(&app_state.pool, &user).await?;

    update_user_favourite_synchonize_time(&app_state.pool, &user)
        .await
        .context("Failed when updating user favourite timestamp")
        .map_err(Error::UnexpectedError)?;

    if latest_favourites_package == favourites_package {
        return Err(Error::ContentEqual(anyhow::anyhow!("Content Equal")));
    }

    Ok(Json(latest_favourites_package))
}

#[tracing::instrument(name = "Write to file", skip_all)]
async fn write_to_file(favourite_package: &FavouritePackage) -> Result<(), anyhow::Error> {
    let file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .append(false)
        .open("favourite_package.json")
        .context("Failed to open file")?;
    serde_json::to_writer_pretty(file, favourite_package).context("Failed to write")?;
    Ok(())
}

#[tracing::instrument(
    name = "update user favourite synchronize time",
    skip(pool, user),
    fields(user_id=user.id)
)]
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
    name = "upsert user categories",
    skip(transaction, user, categories),
    fields(user_id=user.id),
)]
async fn upsert_user_categories(
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
    name = "upsert user favourite manga",
    skip(transaction, user, favourites),
    fields(user_id=user.id)
)]
async fn upsert_user_favourite_manga(
    transaction: &mut Transaction<'_, MySql>,
    user: &User,
    favourites: &Vec<Favourite>,
) -> Result<(), sqlx::Error> {
    for favourite in favourites {
        upsert_manga(transaction, &favourite.manga).await?;

        let query = sqlx::query!(
            r#"
            INSERT INTO favourites
                (manga_id, category_id, sort_key, created_at, deleted_at, user_id)
            VALUES
                (?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY
            UPDATE
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
    }

    Ok(())
}

#[tracing::instrument(
    name = "upsert manga",
    skip(transaction, manga),
    fields(manga_id = manga.manga_id)
)]
pub async fn upsert_manga(
    transaction: &mut Transaction<'_, MySql>,
    manga: &Manga,
) -> Result<(), sqlx::Error> {
    // Helper function to truncate strings
    fn truncate(s: &str, max_len: usize) -> String {
        s.chars().take(max_len).collect()
    }

    // Helper function to truncate optional strings
    fn truncate_opt(s: &Option<String>, max_len: usize) -> Option<String> {
        s.clone().map(|d| truncate(&d, max_len))
    }

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
                title = VALUES(title),
                alt_title = VALUES(alt_title),
                url = VALUES(url),
                public_url = VALUES(public_url),
                rating = VALUES(rating),
                is_nsfw = VALUES(is_nsfw),
                cover_url = VALUES(cover_url),
                large_cover_url = VALUES(large_cover_url),
                state = VALUES(state),
                author = VALUES(author),
                source = VALUES(source)
            "#,
        manga.manga_id,
        truncate(&manga.title, 84),
        truncate_opt(&manga.alt_title, 84),
        truncate(&manga.url, 255),
        truncate(&manga.public_url, 255),
        manga.rating,
        manga.nsfw,
        truncate(&manga.cover_url, 255),
        truncate_opt(&manga.large_cover_url, 84),
        truncate_opt(&manga.state, 24),
        truncate_opt(&manga.author, 32),
        truncate(&manga.source, 32)
    );

    let manga_result = transaction.execute(query).await?;
    if manga_result.rows_affected() > 0 {
        upsert_manga_tags(transaction, manga, &manga.tags).await?;
    }

    Ok(())
}

#[tracing::instrument(
    name = "upsert manga tags",
    skip(transaction, manga, tags),
    fields(manga_id=manga.manga_id)
)]
async fn upsert_manga_tags(
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

        let tag_result = transaction.execute(query).await?;

        if tag_result.rows_affected() > 0 {
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
    }
    Ok(())
}

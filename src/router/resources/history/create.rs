use std::sync::Arc;

use anyhow::Context;
use axum::{Extension, Json, extract::State};
use sqlx::{Executor, MySql, MySqlPool, Transaction};

use crate::{
    authorization::{User, UserId},
    error::Error,
    model::{History, HistoryPackage},
    router::resources::favourites::upsert_manga,
    startup::AppState,
};

use super::index::get_user_history_package;

#[tracing::instrument(
    name = "post history route",
    skip(app_state, user, history_package),
    fields(user_id=user.0)
)]
pub async fn post_history_route(
    State(app_state): State<Arc<AppState>>,
    Extension(user): Extension<UserId>,
    axum::extract::Json(history_package): axum::extract::Json<HistoryPackage>,
) -> Result<Json<HistoryPackage>, Error> {
    let user = user
        .to_user(&app_state.pool)
        .await
        .context("User is missing")
        .map_err(Error::UnexpectedError)?;

    let user = match user {
        Some(user) => user,
        None => return Err(Error::InvalidCredential(anyhow::anyhow!("User not found"))),
    };

    let mut transaction = app_state
        .pool
        .begin()
        .await
        .context("Failed when creating database transaction")
        .map_err(Error::UnexpectedError)?;

    upsert_user_history_manga(&mut transaction, &user, &history_package.history)
        .await
        .context("Failed when upserting history")
        .map_err(Error::UnexpectedError)?;

    transaction
        .commit()
        .await
        .context("Failed when committing transaction")
        .map_err(Error::UnexpectedError)?;

    let latest_history_package = get_user_history_package(&app_state.pool, &user).await?;

    update_user_history_synchronize_time(&app_state.pool, &user)
        .await
        .context("Failed when updating user history timestamp")
        .map_err(Error::UnexpectedError)?;

    if latest_history_package == history_package {
        return Err(Error::ContentEqual(anyhow::anyhow!("Content Equal")));
    }

    Ok(Json(latest_history_package))
}

#[tracing::instrument(
    name = "update user history synchronize time",
    skip(pool, user),
    fields(user_id=user.id)
)]
async fn update_user_history_synchronize_time(
    pool: &MySqlPool,
    user: &User,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now();

    sqlx::query!(
        r#"
        UPDATE users
        SET history_sync_timestamp = ?
        WHERE id = ?
        "#,
        now.timestamp(),
        user.id
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[tracing::instrument(
    name = "upsert user history manga",
    skip(transaction, user, histories),
    fields(user_id=user.id)
)]
async fn upsert_user_history_manga(
    transaction: &mut Transaction<'_, MySql>,
    user: &User,
    histories: &Vec<History>,
) -> Result<(), sqlx::Error> {
    for history in histories {
        upsert_manga(transaction, &history.manga).await?;

        let query = sqlx::query!(
            r#"
                INSERT INTO history
                    (manga_id, created_at, updated_at, chapter_id, page, scroll, percent, chapters, deleted_at, user_id)
                VALUES
                    (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON DUPLICATE KEY UPDATE
                    created_at = ?,
                    updated_at = ?,
                    chapter_id = ?,
                    page = ?,
                    scroll = ?,
                    percent = ?,
                    chapters = ?,
                    deleted_at = ?
                "#,
            history.manga_id,
            history.created_at,
            history.updated_at,
            history.chapter_id,
            history.page,
            history.scroll,
            history.percent,
            history.chapters,
            history.deleted_at,
            user.id,
            history.created_at,
            history.updated_at,
            history.chapter_id,
            history.page,
            history.scroll,
            history.percent,
            history.chapters,
            history.deleted_at,
        );

        transaction.execute(query).await?;
    }

    Ok(())
}

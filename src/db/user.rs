use anyhow::Context;
use secrecy::SecretString;
use sqlx::PgPool;

use crate::{
    auth::{compute_password_hash, error::AuthError},
    error::Error,
    model::User,
    telemetry::spawn_blocking_with_tracing,
};

use super::{PostgresTransaction, error::DatabaseError};

#[tracing::instrument(
    name = "get or create user",
    skip_all,
    fields(email, allow_registration)
)]
pub async fn get_or_create_user(
    pool: &PgPool,
    email: String,
    password: SecretString,
    allow_registration: bool,
) -> Result<(User, String), Error> {
    let user_option = sqlx::query!(
        r#"
        SELECT 
            id, email, password, nickname
        FROM
            users
        WHERE 
            email = $1
    "#,
        email
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| Error::Database(DatabaseError::DatabaseError(e)))?;

    if user_option.is_some() {
        let row = user_option.unwrap();

        return Ok((
            User {
                id: row.id,
                email: row.email,
                nickname: row.nickname,
            },
            row.password,
        ));
    }

    if !allow_registration {
        return Err(Error::Auth(AuthError::UserNotFound));
    }

    create_user(pool, email, password).await
}

#[tracing::instrument(name = "create user", skip_all, fields(email))]
pub async fn create_user(
    pool: &PgPool,
    email: String,
    password: SecretString,
) -> Result<(User, String), Error> {
    let password_hashed = spawn_blocking_with_tracing(move || compute_password_hash(password))
        .await
        .context("compute password hash")
        .map_err(Error::Other)??;

    let user_id = sqlx::query!(
        r#"
        INSERT INTO USERS 
            (email, password)
        VALUES 
            ($1, $2)
        RETURNING id;
    "#,
        email,
        password_hashed
    )
    .fetch_one(pool)
    .await
    .map_err(|e| Error::Database(DatabaseError::DatabaseError(e)))?;

    Ok((
        User {
            id: user_id.id,
            email,
            nickname: None,
        },
        password_hashed,
    ))
}

#[tracing::instrument(name = "get user by id", skip_all, fields(user_id))]
pub async fn get_user_by_id_optional(pool: &PgPool, user_id: i64) -> Result<Option<User>, Error> {
    sqlx::query_as!(
        User,
        r#"
        SELECT 
            id, email, nickname 
        FROM 
            users
        WHERE 
            id = $1;
    "#,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| Error::Database(crate::db::error::DatabaseError::DatabaseError(e)))
}

pub async fn update_user_history_sync_time(
    tx: &mut PostgresTransaction,
    user_id: i64,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        UPDATE users
        SET
            history_sync_timestamp = $1
        WHERE 
            id = $2;
    "#,
        // sync_time,
        chrono::Utc::now().timestamp_millis(),
        user_id
    )
    .execute(&mut **tx)
    .await
    .map_err(DatabaseError::DatabaseError)?;

    Ok(())
}

pub async fn update_user_favourite_sync_time(
    tx: &mut PostgresTransaction,
    user_id: i64,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        UPDATE users
        SET
            favourites_sync_timestamp = $1
        WHERE 
            id = $2;
    "#,
        // sync_time,
        chrono::Utc::now().timestamp_millis(),
        user_id
    )
    .execute(&mut **tx)
    .await
    .map_err(DatabaseError::DatabaseError)?;

    Ok(())
}

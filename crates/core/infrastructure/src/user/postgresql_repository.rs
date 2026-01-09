use async_trait::async_trait;
use chrono::Utc;
use core_domain::{
    shared::error::DomainError,
    user::{
        model::{User, UserCreate, UserPasswordReset, UserUpdatePassword, UserUpdateSyncTime},
        repository::UserRepository,
    },
};
use sqlx::PgPool;
use tracing::{Level, info, instrument};

pub struct PostgreSQLUserRepository {
    pub pool: PgPool,
}

#[async_trait]
impl UserRepository for PostgreSQLUserRepository {
    #[instrument(name = "repository::get_user_by_email", skip(self), level = Level::DEBUG, err(level = Level::ERROR))]
    async fn get_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        sqlx::query_as!(
            User,
            r#"
                SELECT 
                    *
                FROM
                    users
                WHERE 
                    email = $1;
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.into()))
    }

    #[instrument(name = "repository::get_user_by_id", skip(self), level = Level::DEBUG, err(level = Level::ERROR))]
    async fn get_by_id(&self, id: i64) -> Result<Option<User>, DomainError> {
        sqlx::query_as!(
            User,
            r#"
                SELECT 
                    *
                FROM
                    users
                WHERE 
                    id = $1;
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.into()))
    }

    #[instrument(name = "repository::create_user", skip_all, level = Level::DEBUG, err(level = Level::ERROR))]
    async fn create(&self, data: UserCreate) -> Result<User, DomainError> {
        info!("inserting user ({}) to database", data.email);

        let user_id = sqlx::query!(
            r#"
                INSERT INTO USERS 
                    (email, password, nickname)
                VALUES 
                    ($1, $2, $3)
                RETURNING id;
            "#,
            data.email,
            data.password,
            data.nickname
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.into()))?;

        Ok(User {
            id: user_id.id,
            email: data.email,
            password: data.password,
            nickname: data.nickname,
            favourites_sync_timestamp: None,
            history_sync_timestamp: None,
            password_reset_token_hash: None,
            password_reset_token_expires_at: None,
        })
    }

    #[instrument(name = "repository::update_user_favourite_sync_time", skip_all, level = Level::DEBUG, err(level = Level::ERROR))]
    async fn update_favourite_sync_time(
        &self,
        data: UserUpdateSyncTime,
    ) -> Result<(), DomainError> {
        info!(
            "update user({}) favourite sync timestamp to {}",
            data.user_id, data.time
        );

        sqlx::query!(
            r#"
                UPDATE users
                SET
                    favourites_sync_timestamp = $1
                WHERE 
                    id = $2;
            "#,
            data.time,
            data.user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.into()))?;

        Ok(())
    }

    #[instrument(name = "repository::update_user_history_sync_time", skip_all, level = Level::DEBUG, err(level = Level::ERROR))]
    async fn update_history_sync_time(&self, data: UserUpdateSyncTime) -> Result<(), DomainError> {
        info!(
            "update user({}) history sync timestamp to {}",
            data.user_id, data.time
        );

        sqlx::query!(
            r#"
                UPDATE users
                SET
                    history_sync_timestamp = $1
                WHERE 
                    id = $2;
            "#,
            data.time,
            data.user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.into()))?;

        Ok(())
    }

    #[instrument(name = "repository::update_user_password_reset_token", skip_all, level=Level::DEBUG, err(level=Level::ERROR))]
    async fn update_password_reset_token(
        &self,
        data: UserPasswordReset,
    ) -> Result<(), DomainError> {
        sqlx::query!(
            r#"
                UPDATE users
                SET
                    password_reset_token_hash = $1,
                    password_reset_token_expires_at = $2
                WHERE id = $3;
            "#,
            data.token,
            data.expires_at,
            data.user_id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.into()))?;

        Ok(())
    }

    #[instrument(name = "repository::update_user_password", skip_all, level = Level::DEBUG, err(level = Level::ERROR))]
    async fn update_user_password(&self, data: UserUpdatePassword) -> Result<(), DomainError> {
        sqlx::query!(
            r#"
                UPDATE users
                SET 
                    password = $1,
                    password_reset_token_hash = NULL,
                    password_reset_token_expires_at = NULL
                WHERE id = $2;
            "#,
            data.password,
            data.user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.into()))?;

        Ok(())
    }

    #[instrument(name = "repository::list_users_by_active_password_reset_token", skip_all, level = Level::DEBUG, err(level = Level::ERROR))]
    async fn list_by_active_password_reset_token(&self) -> Result<Vec<User>, DomainError> {
        let now = Utc::now().timestamp();

        sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users
            WHERE password_reset_token_expires_at IS NOT NULL
            AND password_reset_token_expires_at > $1;
            "#,
            now
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.into()))
    }
}

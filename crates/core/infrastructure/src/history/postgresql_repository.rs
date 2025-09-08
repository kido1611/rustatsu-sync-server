use async_trait::async_trait;
use core_domain::{
    history::{model::History, repository::HistoryRepository},
    shared::error::DomainError,
};
use sqlx::{PgPool, Postgres, QueryBuilder};
use tracing::{info, instrument};

pub struct PostgreSQLHistoryRepository {
    pub pool: PgPool,
}

#[async_trait]
impl HistoryRepository for PostgreSQLHistoryRepository {
    #[instrument(name = "repository::insert_history", skip_all)]
    async fn insert(&self, history: Vec<History>) -> Result<(), DomainError> {
        info!("inserting {} history", history.len());

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::DatabaseError(e.into()))?;

        for chunk in history.chunks(300) {
            let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
                r#"
                    INSERT INTO history 
                        (manga_id, created_at, updated_at, chapter_id, page, scroll, percent, chapters, deleted_at, user_id)"#,
            );

            builder.push_values(chunk, |mut b, history_item| {
                b.push_bind(history_item.manga_id)
                    .push_bind(history_item.created_at)
                    .push_bind(history_item.updated_at)
                    .push_bind(history_item.chapter_id)
                    .push_bind(history_item.page)
                    .push_bind(history_item.scroll)
                    .push_bind(history_item.percent)
                    .push_bind(history_item.chapters)
                    .push_bind(history_item.deleted_at)
                    .push_bind(history_item.user_id);
            });

            builder.push(
                r#" 
                    ON CONFLICT (manga_id, user_id)
                    DO UPDATE SET 
                        created_at = EXCLUDED.created_at,
                        updated_at = EXCLUDED.updated_at,
                        chapter_id = EXCLUDED.chapter_id,
                        page = EXCLUDED.page,
                        scroll = EXCLUDED.scroll,
                        percent = EXCLUDED.percent,
                        chapters = EXCLUDED.chapters,
                        deleted_at = EXCLUDED.deleted_at;
                "#,
            );

            builder
                .build()
                .execute(&mut *tx)
                .await
                .map_err(|e| DomainError::DatabaseError(e.into()))?;
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::DatabaseError(e.into()))?;

        Ok(())
    }

    #[instrument(name = "repository::list_user_history", skip(self))]
    async fn list_by_user_id(&self, user_id: i64) -> Result<Vec<History>, DomainError> {
        let history = sqlx::query_as!(
            History,
            r#"
                SELECT 
                    manga_id, created_at, updated_at,
                    chapter_id, page, scroll,
                    percent, chapters, deleted_at,
                    user_id
                FROM
                    history
                WHERE user_id = $1;
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.into()))?;

        info!("user have {} history", history.len());

        Ok(history)
    }
}

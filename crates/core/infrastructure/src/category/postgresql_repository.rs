use async_trait::async_trait;
use core_domain::{
    category::{model::Category, repository::CategoryRepository},
    shared::error::DomainError,
};
use sqlx::{PgPool, Postgres, QueryBuilder};
use tracing::{info, instrument};

pub struct PostgreSQLCategoryRepository {
    pub pool: PgPool,
}

#[async_trait]
impl CategoryRepository for PostgreSQLCategoryRepository {
    #[instrument(name = "repository::insert_categories", skip_all)]
    async fn insert(&self, categories: Vec<Category>) -> Result<(), DomainError> {
        info!("inserting {} categories", categories.len());

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::DatabaseError(e.into()))?;

        for chunk in categories.chunks(300) {
            let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
                r#"INSERT INTO categories (id, created_at, sort_key, title, "order", user_id, track, show_in_lib, deleted_at)"#,
            );

            builder.push_values(chunk, |mut b, category| {
                b.push_bind(category.id)
                    .push_bind(category.created_at)
                    .push_bind(category.sort_key)
                    .push_bind(&category.title)
                    .push_bind(&category.order)
                    .push_bind(category.user_id)
                    .push_bind(category.track)
                    .push_bind(category.show_in_lib)
                    .push_bind(category.deleted_at);
            });

            builder.push(
                r#" 
                ON CONFLICT (id, user_id)
                DO UPDATE SET 
                    created_at = EXCLUDED.created_at,
                    sort_key = EXCLUDED.sort_key,
                    title = EXCLUDED.title,
                    "order" = EXCLUDED.order,
                    track = EXCLUDED.track,
                    show_in_lib = EXCLUDED.show_in_lib,
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

    #[instrument(name = "repository::list_user_categories", skip(self))]
    async fn list_by_user_id(&self, user_id: i64) -> Result<Vec<Category>, DomainError> {
        let categories = sqlx::query_as!(
            Category,
            r#"
                SELECT id, created_at, sort_key, title, "order", track, user_id, show_in_lib, deleted_at
                FROM categories
                WHERE user_id = $1;
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.into()))?;

        info!("user have {} categories", categories.len());

        Ok(categories)
    }
}

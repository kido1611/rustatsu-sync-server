use async_trait::async_trait;
use core_domain::{
    favourite::{model::Favourite, repository::FavouriteRepository},
    shared::error::DomainError,
};
use sqlx::{PgPool, Postgres, QueryBuilder};
use tracing::{info, instrument};

pub struct PostgreSQLFavouriteRepository {
    pub pool: PgPool,
}

#[async_trait]
impl FavouriteRepository for PostgreSQLFavouriteRepository {
    #[instrument(name = "repository::insert_favourites", skip_all)]
    async fn insert(&self, favourites: Vec<Favourite>) -> Result<(), DomainError> {
        info!("inserting {} favourites", favourites.len());

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::DatabaseError(e.into()))?;

        for chunk in favourites.chunks(300) {
            let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
                r#"INSERT INTO favourites (manga_id, category_id, user_id, sort_key, created_at, deleted_at, pinned)"#,
            );

            builder.push_values(chunk, |mut b, favourite| {
                b.push_bind(favourite.manga_id)
                    .push_bind(favourite.category_id)
                    .push_bind(favourite.user_id)
                    .push_bind(favourite.sort_key)
                    .push_bind(favourite.created_at)
                    .push_bind(favourite.deleted_at)
                    .push_bind(favourite.pinned);
            });

            builder.push(
                r#" 
                ON CONFLICT (manga_id, user_id, category_id)
                DO UPDATE SET 
                    created_at = EXCLUDED.created_at,
                    sort_key = EXCLUDED.sort_key,
                    deleted_at = EXCLUDED.deleted_at,
                    pinned = EXCLUDED.pinned;
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

    #[instrument(name = "repository::list_user_favourites", skip(self))]
    async fn list_by_user_id(&self, user_id: i64) -> Result<Vec<Favourite>, DomainError> {
        let favourites = sqlx::query_as!(
            Favourite,
            r#"
                SELECT manga_id, category_id, user_id, sort_key, created_at, deleted_at, pinned
                FROM favourites
                WHERE user_id = $1;
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.into()))?;

        info!("user have {} favourites", favourites.len());

        Ok(favourites)
    }
}

use async_trait::async_trait;
use core_domain::{
    manga_tag::{model::MangaTag, repository::MangaTagRepository},
    shared::error::DomainError,
};
use sqlx::{PgPool, Postgres, QueryBuilder};
use tracing::{info, instrument};

pub struct PostgreSQLMangaTagRepository {
    pub pool: PgPool,
}

#[async_trait]
impl MangaTagRepository for PostgreSQLMangaTagRepository {
    #[instrument(name = "repository::insert_manga_tags", skip_all)]
    async fn insert(&self, manga_tags: Vec<MangaTag>) -> Result<(), DomainError> {
        info!("inserting {} manga tags", manga_tags.len());

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::DatabaseError(e.into()))?;

        for chunk in manga_tags.chunks(1000) {
            let mut builder: QueryBuilder<Postgres> =
                QueryBuilder::new(r#"INSERT INTO manga_tags (manga_id, tag_id)"#);

            builder.push_values(chunk, |mut b, manga_tag| {
                b.push_bind(manga_tag.manga_id).push_bind(manga_tag.tag_id);
            });

            builder.push(
                r#" 
                    ON CONFLICT (manga_id, tag_id)
                    DO NOTHING;
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
}

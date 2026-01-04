use async_trait::async_trait;
use core_domain::{
    shared::error::DomainError,
    tag::{model::Tag, repository::TagRepository},
};
use futures::TryStreamExt;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};
use tracing::{Level, info, instrument};

pub struct PostgreSQLTagRepository {
    pub pool: PgPool,
}

#[async_trait]
impl TagRepository for PostgreSQLTagRepository {
    #[instrument(name = "repository::insert_tags", skip_all, level = Level::DEBUG, err(level = Level::ERROR))]
    async fn insert(&self, tags: Vec<Tag>) -> Result<(), DomainError> {
        info!("inserting {} tags", tags.len());

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::DatabaseError(e.into()))?;

        for chunk in tags.chunks(300) {
            let mut builder: QueryBuilder<Postgres> =
                QueryBuilder::new(r#"INSERT INTO tags (id, title, key, source)"#);

            builder.push_values(chunk, |mut b, tag| {
                b.push_bind(tag.id)
                    .push_bind(&tag.title)
                    .push_bind(&tag.key)
                    .push_bind(&tag.source);
            });

            builder.push(
                r#" ON CONFLICT (id)
                    DO UPDATE SET
                        title = EXCLUDED.title,
                        key = EXCLUDED.key,
                        source = EXCLUDED.source;
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

    #[instrument(name = "repository::list_tags_by_manga_ids", skip_all, fields(manga_count = %manga_ids.len()), level = Level::DEBUG, err(level = Level::ERROR))]
    async fn list_by_manga_ids(&self, manga_ids: &[i64]) -> Result<Vec<(i64, Tag)>, DomainError> {
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
                SELECT 
                    manga_tags.manga_id, tags.id, tags.title, tags."key", tags.source
                FROM manga_tags
                INNER JOIN tags ON manga_tags.tag_id = tags.id
                WHERE manga_tags.manga_id in (
            "#,
        );
        let mut builder_value_separator = builder.separated(", ");
        for id in manga_ids {
            builder_value_separator.push_bind(id);
        }
        builder_value_separator.push_unseparated(");");

        let mut tags_stream = builder.build().fetch(&self.pool);

        let mut manga_tags = Vec::new();
        while let Some(row) = tags_stream
            .try_next()
            .await
            .map_err(|e| DomainError::DatabaseError(e.into()))?
        {
            let manga_id: i64 = row.get("manga_id");
            manga_tags.push((
                manga_id,
                Tag {
                    id: row.get("id"),
                    title: row.get("title"),
                    key: row.get("key"),
                    source: row.get("source"),
                },
            ));
        }

        Ok(manga_tags)
    }
}

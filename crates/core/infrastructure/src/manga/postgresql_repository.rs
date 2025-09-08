use async_trait::async_trait;
use core_domain::{
    manga::{
        model::{Manga, MangaPagination},
        repository::MangaRepository,
    },
    shared::error::DomainError,
};
use futures::TryStreamExt;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};
use tracing::{info, instrument};

pub struct PostgreSQLMangaRepository {
    pub pool: PgPool,
}

#[async_trait]
impl MangaRepository for PostgreSQLMangaRepository {
    #[instrument(name = "repository::insert_manga", skip_all)]
    async fn insert(&self, manga: Vec<Manga>) -> Result<(), DomainError> {
        info!("inserting {} manga", manga.len());

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::DatabaseError(e.into()))?;

        for chunk in manga.chunks(300) {
            let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
                r#"INSERT INTO mangas (id, title, alt_title, url, public_url, rating, content_rating, cover_url, large_cover_url, state, author, source)"#,
            );

            builder.push_values(chunk, |mut b, m| {
                b.push_bind(m.id)
                    .push_bind(&m.title)
                    .push_bind(&m.alt_title)
                    .push_bind(&m.url)
                    .push_bind(&m.public_url)
                    .push_bind(m.rating)
                    .push_bind(&m.content_rating)
                    .push_bind(&m.cover_url)
                    .push_bind(&m.large_cover_url)
                    .push_bind(&m.state)
                    .push_bind(&m.author)
                    .push_bind(&m.source);
            });

            builder.push(
                r#" ON CONFLICT (id)
                    DO UPDATE SET
                        title = EXCLUDED.title, 
                        alt_title = EXCLUDED.alt_title, 
                        url = EXCLUDED.url, 
                        public_url = EXCLUDED.public_url, 
                        rating = EXCLUDED.rating, 
                        content_rating = EXCLUDED.content_rating, 
                        cover_url = EXCLUDED.cover_url, 
                        large_cover_url = EXCLUDED.large_cover_url, 
                        state = EXCLUDED.state, 
                        author = EXCLUDED.author, 
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

    #[instrument(name = "repository::get_manga_by_id", skip_all, filds(manga_id = %id))]
    async fn get_by_id(&self, id: i64) -> Result<Option<Manga>, DomainError> {
        sqlx::query_as!(
            Manga,
            r#"
                SELECT
                    id, title, alt_title,
                    url, public_url, rating,
                    content_rating, cover_url, large_cover_url,
                    state, author, source
                FROM
                    mangas
                WHERE
                    id = $1;
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.into()))
    }

    #[instrument(name = "repository::list_manga_by_ids", skip_all)]
    async fn list_by_ids(&self, ids: &[i64]) -> Result<Vec<Manga>, DomainError> {
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
                SELECT 
                    id, title, alt_title, url, public_url, rating, content_rating, cover_url, large_cover_url, state, author, source
                FROM mangas
                WHERE id in (
            "#,
        );
        let mut builder_value_separator = builder.separated(", ");
        for id in ids.iter() {
            builder_value_separator.push_bind(id);
        }
        builder_value_separator.push_unseparated(");");

        let mut manga_stream = builder.build().fetch(&self.pool);

        let mut manga = Vec::new();
        while let Some(row) = manga_stream
            .try_next()
            .await
            .map_err(|e| DomainError::DatabaseError(e.into()))?
        {
            manga.push(Manga {
                id: row.get("id"),
                title: row.get("title"),
                alt_title: row.get("alt_title"),
                url: row.get("url"),
                public_url: row.get("public_url"),
                rating: row.get("rating"),
                content_rating: row.get("content_rating"),
                cover_url: row.get("cover_url"),
                large_cover_url: row.get("large_cover_url"),
                state: row.get("state"),
                author: row.get("author"),
                source: row.get("source"),
            });
        }

        Ok(manga)
    }

    #[instrument(name = "repository::list_manga", skip_all, fields(offset = %pagination.offset, limit = %pagination.limit))]
    async fn list_with_pagination(
        &self,
        pagination: MangaPagination,
    ) -> Result<Vec<Manga>, DomainError> {
        sqlx::query_as!(
            Manga,
            r#"
                SELECT
                    id, title, alt_title,
                    url, public_url, rating,
                    content_rating, cover_url, large_cover_url,
                    state, author, source
                FROM
                    mangas
                ORDER BY id
                LIMIT $1
                OFFSET $2
            "#,
            pagination.limit,
            pagination.offset,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.into()))
    }
}

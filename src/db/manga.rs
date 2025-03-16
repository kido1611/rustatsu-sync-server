use std::sync::Arc;

use futures::TryStreamExt;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};

use crate::{
    error::Error,
    model::{Manga, MangaTag, Tag},
};

use super::error::DatabaseError;

#[tracing::instrument(name = "get manga with pagination", skip_all)]
pub async fn get_manga_with_pagination(
    pool: &PgPool,
    limit: i64,
    skip: i64,
) -> Result<Vec<Manga>, Error> {
    let manga_raw = sqlx::query!(
        r#"
        SELECT
            id, title, alt_title,
            url, public_url, rating,
            is_nsfw, cover_url, large_cover_url,
            state, author, source
        FROM
            mangas
        ORDER BY id
        LIMIT $1
        OFFSET $2
    "#,
        limit,
        skip
    )
    .fetch_all(pool)
    .await
    .map_err(DatabaseError::DatabaseError)?;

    let mut tag_query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT
            manga_tags.manga_id, tags.id, tags.title, tags."key", tags.source
        FROM
            manga_tags
        INNER JOIN
            tags ON manga_tags.tag_id = tags.id
        WHERE
            manga_tags.manga_id in (
    "#,
    );
    let mut tag_query_builder_separator = tag_query_builder.separated(", ");
    for manga in &manga_raw {
        tag_query_builder_separator.push_bind(manga.id);
    }
    tag_query_builder_separator.push_unseparated(");");
    let mut tag_stream = tag_query_builder.build().fetch(pool);

    let mut manga_tags = Vec::new();
    while let Some(row) = tag_stream
        .try_next()
        .await
        .map_err(DatabaseError::DatabaseError)?
    {
        manga_tags.push(MangaTag {
            manga_id: row.get("manga_id"),
            tag: Arc::new(Tag {
                tag_id: row.get("id"),
                title: row.get("title"),
                key: row.get("key"),
                source: row.get("source"),
            }),
        });
    }

    let mut mangas = Vec::new();
    for manga in manga_raw {
        let manga_id: i64 = manga.id;
        let tags = manga_tags
            .iter()
            .filter(|f| f.manga_id == manga_id)
            .map(|m| Arc::clone(&m.tag))
            .collect();

        mangas.push(Manga {
            manga_id: manga.id,
            title: manga.title,
            alt_title: manga.alt_title,
            url: manga.url,
            public_url: manga.public_url,
            rating: manga.rating,
            nsfw: if manga.is_nsfw { Some(1) } else { Some(0) },
            content_rating: if manga.is_nsfw {
                Some("ADULT".to_string())
            } else {
                None
            },
            cover_url: manga.cover_url,
            large_cover_url: manga.large_cover_url,
            state: manga.state,
            author: manga.author,
            source: manga.source,
            tags,
        });
    }

    Ok(mangas)
}

#[tracing::instrument(name = "get manga by id", skip_all, fields(manga_id))]
pub async fn get_manga_by_id(pool: &PgPool, manga_id: i64) -> Result<Manga, Error> {
    let manga_raw = match sqlx::query!(
        r#"
        SELECT
            id, title, alt_title,
            url, public_url, rating,
            is_nsfw, cover_url, large_cover_url,
            state, author, source
        FROM
            mangas
        WHERE
            id = $1;
    "#,
        manga_id
    )
    .fetch_optional(pool)
    .await
    .map_err(DatabaseError::DatabaseError)?
    {
        Some(manga) => manga,
        None => {
            return Err(Error::Database(DatabaseError::NotFound));
        }
    };

    let tags = sqlx::query!(
        r#"
        SELECT
            tags.id, tags.title, tags."key", tags.source
        FROM
            manga_tags
        INNER JOIN
            tags ON manga_tags.tag_id = tags.id
        WHERE 
            manga_tags.manga_id = $1; 
    "#,
        manga_id
    )
    .map(|row| {
        Arc::new(Tag {
            tag_id: row.id,
            title: row.title,
            key: row.key,
            source: row.source,
        })
    })
    .fetch_all(pool)
    .await
    .map_err(DatabaseError::DatabaseError)?;

    Ok(Manga {
        manga_id: manga_raw.id,
        title: manga_raw.title,
        alt_title: manga_raw.alt_title,
        url: manga_raw.url,
        public_url: manga_raw.public_url,
        rating: manga_raw.rating,
        nsfw: if manga_raw.is_nsfw { Some(1) } else { Some(0) },
        content_rating: if manga_raw.is_nsfw {
            Some("ADULT".to_string())
        } else {
            None
        },
        cover_url: manga_raw.cover_url,
        large_cover_url: manga_raw.large_cover_url,
        state: manga_raw.state,
        author: manga_raw.author,
        source: manga_raw.source,
        tags,
    })
}

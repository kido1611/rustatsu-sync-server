use std::sync::Arc;

use futures::TryStreamExt;
use sqlx::{PgPool, Postgres, QueryBuilder, Row, Transaction};

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

    if manga_raw.is_empty() {
        return Ok(Vec::new());
    }

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

pub async fn insert_mangas(
    tx: &mut Transaction<'_, Postgres>,
    data: &[Arc<Manga>],
) -> Result<(), Error> {
    for manga in data {
        let is_nsfw = match manga.nsfw {
            Some(val) => {
                if val > 0 {
                    true
                } else {
                    match &manga.content_rating {
                        Some(val) => val.to_lowercase() == "adult",
                        None => false,
                    }
                }
            }
            None => match &manga.content_rating {
                Some(val) => val.to_lowercase() == "adult",
                None => false,
            },
        };
        let author = match &manga.author {
            Some(val) => {
                let mut author = val.clone();
                author.truncate(120);
                Some(author)
            }
            None => None,
        };

        sqlx::query!(
            r#"
            INSERT INTO mangas
                (id, title, alt_title, url, public_url, rating, is_nsfw, cover_url, large_cover_url, state, author, source)
            VALUES 
                ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (id)
            DO UPDATE SET
                title = $2, 
                alt_title = $3, 
                url = $4, 
                public_url = $5, 
                rating = $6, 
                is_nsfw = $7, 
                cover_url = $8, 
                large_cover_url = $9, 
                state = $10, 
                author = $11, 
                source = $12;
        "#,
            manga.manga_id,
            manga.title,
            manga.alt_title,
            manga.url,
            manga.public_url,
            manga.rating,
            is_nsfw,
            manga.cover_url,
            manga.large_cover_url,
            manga.state,
            author,
            manga.source
        )
        .execute(&mut **tx)
        .await
        .map_err(DatabaseError::DatabaseError)?;
    }

    Ok(())
}

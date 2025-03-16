use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use futures::TryStreamExt;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};

use crate::{
    db::error::DatabaseError,
    error::Error,
    model::{History, Manga, MangaTag, MangaTagEntity, Tag, UserHistory},
};

#[tracing::instrument(name = "get user_history", skip_all)]
pub async fn get_user_history(pool: &PgPool, user_id: i64) -> Result<UserHistory, Error> {
    let raw_history = sqlx::query!(
        r#"
        SELECT 
            manga_id
        FROM
            history
        WHERE
            user_id = $1
    "#,
        user_id
    )
    .fetch_all(pool)
    .await
    .map_err(DatabaseError::DatabaseError)?;

    if raw_history.is_empty() {
        return Ok(UserHistory {
            timestamp: chrono::Utc::now().timestamp(),
            history: vec![],
        });
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
    for history in &raw_history {
        tag_query_builder_separator.push_bind(history.manga_id);
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

    let mut manga_query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT
            id, title, alt_title,
            url, public_url, rating,
            is_nsfw, cover_url, large_cover_url,
            state, author, source
        FROM
            mangas
        WHERE 
            id in (
    "#,
    );
    let mut manga_query_builder_separator = manga_query_builder.separated(", ");
    for history in &raw_history {
        manga_query_builder_separator.push_bind(history.manga_id);
    }
    manga_query_builder_separator.push_unseparated(");");
    let mut manga_stream = manga_query_builder.build().fetch(pool);

    let mut mangas = Vec::new();
    while let Some(row) = manga_stream
        .try_next()
        .await
        .map_err(DatabaseError::DatabaseError)?
    {
        let manga_id: i64 = row.get("id");
        let tags = manga_tags
            .iter()
            .filter(|f| f.manga_id == manga_id)
            .map(|m| Arc::clone(&m.tag))
            .collect();

        mangas.push(Arc::new(Manga {
            manga_id,
            title: row.get("title"),
            alt_title: row.get("alt_title"),
            url: row.get("url"),
            public_url: row.get("public_url"),
            rating: row.get("rating"),
            nsfw: if row.get("is_nsfw") { Some(1) } else { Some(0) },
            content_rating: if row.get("is_nsfw") {
                Some("ADULT".to_string())
            } else {
                None
            },
            cover_url: row.get("cover_url"),
            large_cover_url: row.get("large_cover_url"),
            state: row.get("state"),
            author: row.get("author"),
            source: row.get("source"),
            tags,
        }));
    }

    let mut history_stream = sqlx::query!(
        r#"
        SELECT 
            manga_id, created_at, updated_at,
            chapter_id, page, scroll,
            percent, chapters, deleted_at,
            user_id
        FROM
            history
        WHERE 
            user_id = $1
    "#,
        user_id
    )
    .fetch(pool);

    let mut history = Vec::new();
    while let Some(row) = history_stream
        .try_next()
        .await
        .map_err(DatabaseError::DatabaseError)?
    {
        let manga_id = row.manga_id;
        if let Some(manga) = mangas.iter().find(|m| m.manga_id == manga_id) {
            history.push(History {
                manga_id,
                manga: Arc::clone(manga),
                created_at: row.created_at,
                updated_at: row.updated_at,
                chapter_id: row.chapter_id,
                page: row.page,
                scroll: row.scroll,
                percent: row.percent,
                chapters: row.chapters,
                deleted_at: row.deleted_at,
            });
        } else {
            continue;
        }
    }

    let user = sqlx::query!(
        r#"
        SELECT
            id, history_sync_timestamp
        FROM 
            users
        WHERE
            id = $1
    "#,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(DatabaseError::DatabaseError)?;

    Ok(UserHistory {
        history,
        timestamp: match user.history_sync_timestamp {
            Some(time) => {
                if time == 0 {
                    chrono::Utc::now().timestamp()
                } else {
                    time
                }
            }
            None => chrono::Utc::now().timestamp(),
        },
    })
}

#[tracing::instrument(name = "update user_history", skip_all)]
pub async fn update_user_history(
    pool: &PgPool,
    user_id: i64,
    user_history: UserHistory,
) -> Result<(), Error> {
    let mut mangas_map: HashMap<i64, Arc<Manga>> = HashMap::new();
    let mut tags_map: HashMap<i64, Arc<Tag>> = HashMap::new();
    let mut manga_tags_set = HashSet::new();

    for history in &user_history.history {
        for tag in &history.manga.tags {
            tags_map.insert(tag.tag_id, Arc::clone(tag));

            manga_tags_set.insert(MangaTagEntity {
                manga_id: history.manga_id,
                tag_id: tag.tag_id,
            });
        }
        mangas_map.insert(history.manga_id, Arc::clone(&history.manga));
    }

    let mut tx = pool.begin().await.map_err(DatabaseError::DatabaseError)?;

    for tag in tags_map.values() {
        sqlx::query!(
            r#"
            INSERT INTO tags 
                (id, title, "key", source)
            VALUES
                ($1, $2, $3, $4)
            ON CONFLICT (id)
            DO UPDATE SET
                title = $2,
                "key" = $3,
                source = $4;
        "#,
            tag.tag_id,
            tag.title,
            tag.key,
            tag.source,
        )
        .execute(&mut *tx)
        .await
        .map_err(DatabaseError::DatabaseError)?;
    }

    for manga in mangas_map.values() {
        let is_nsfw = match manga.nsfw {
            Some(_) => true,
            None => match &manga.content_rating {
                Some(val) => val.to_lowercase() == "adult",
                None => false,
            },
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
            manga.author,
            manga.source
        )
        .execute(&mut *tx)
        .await
        .map_err(DatabaseError::DatabaseError)?;
    }

    for batch in Vec::from_iter(manga_tags_set.iter()).chunks(300) {
        let mut manga_tag_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            INSERT INTO manga_tags
                (manga_id, tag_id)
        "#,
        );

        manga_tag_builder.push_values(batch, |mut b, manga_tag| {
            b.push_bind(manga_tag.manga_id).push_bind(manga_tag.tag_id);
        });
        manga_tag_builder.push(" ON CONFLICT (manga_id, tag_id) DO NOTHING;");

        manga_tag_builder
            .build()
            .execute(&mut *tx)
            .await
            .map_err(DatabaseError::DatabaseError)?;
    }

    for batch in user_history.history.chunks(200) {
        let mut history_query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            INSERT INTO history 
                (manga_id, created_at, updated_at, chapter_id, page, scroll, percent, chapters, deleted_at, user_id)
        "#,
        );

        history_query_builder.push_values(batch, |mut b, his| {
            b.push_bind(his.manga_id)
                .push_bind(his.created_at)
                .push_bind(his.updated_at)
                .push_bind(his.chapter_id)
                .push_bind(his.page)
                .push_bind(his.scroll)
                .push_bind(his.percent)
                .push_bind(his.chapters)
                .push_bind(his.deleted_at)
                .push_bind(user_id);
        });
        history_query_builder.push(
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

        history_query_builder
            .build()
            .execute(&mut *tx)
            .await
            .map_err(DatabaseError::DatabaseError)?;
    }

    sqlx::query!(
        r#"
        UPDATE users
        SET
            history_sync_timestamp = $1
        WHERE 
            id = $2;
    "#,
        chrono::Utc::now().timestamp(),
        user_id
    )
    .execute(&mut *tx)
    .await
    .map_err(DatabaseError::DatabaseError)?;

    tx.commit().await.map_err(DatabaseError::DatabaseError)?;

    drop(tags_map);
    drop(mangas_map);
    drop(manga_tags_set);

    Ok(())
}

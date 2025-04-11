use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use futures::TryStreamExt;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};

use crate::{
    db::error::DatabaseError,
    error::Error,
    model::{Category, Favourite, Manga, MangaTag, MangaTagEntity, Tag, UserFavourite},
};

#[tracing::instrument(name = "get user_favourite", skip_all)]
pub async fn get_user_favourites(pool: &PgPool, user_id: i64) -> Result<UserFavourite, Error> {
    let categories = sqlx::query!(
        r#"
        SELECT
            id, created_at, sort_key, title, "order", track, show_in_lib, deleted_at
        FROM
            categories
        WHERE
            user_id = $1
    "#,
        user_id
    )
    .map(|row| Category {
        category_id: row.id,
        created_at: row.created_at,
        sort_key: row.sort_key,
        track: if row.track { 1 } else { 0 },
        show_in_lib: if row.show_in_lib { 1 } else { 0 },
        deleted_at: row.deleted_at,
        title: row.title,
        order: row.order,
    })
    .fetch_all(pool)
    .await
    .map_err(DatabaseError::DatabaseError)?;

    if categories.is_empty() {
        return Ok(UserFavourite {
            favourite_categories: vec![],
            favourites: vec![],
            timestamp: chrono::Utc::now().timestamp(),
        });
    }

    let raw_favourites = sqlx::query!(
        r#"
        SELECT 
            manga_id
        FROM
            favourites
        WHERE
            user_id = $1
    "#,
        user_id
    )
    .fetch_all(pool)
    .await
    .map_err(DatabaseError::DatabaseError)?;

    if raw_favourites.is_empty() {
        return Ok(UserFavourite {
            favourite_categories: categories,
            favourites: vec![],
            timestamp: chrono::Utc::now().timestamp(),
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
    for favourite in &raw_favourites {
        tag_query_builder_separator.push_bind(favourite.manga_id);
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
    for favourite in &raw_favourites {
        manga_query_builder_separator.push_bind(favourite.manga_id);
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

    let mut favourites_stream = sqlx::query!(
        r#"
        SELECT 
            manga_id, category_id, user_id,
            sort_key, created_at, deleted_at
        FROM
            favourites
        WHERE 
            user_id = $1
    "#,
        user_id
    )
    .fetch(pool);

    let mut favourites = Vec::new();
    while let Some(row) = favourites_stream
        .try_next()
        .await
        .map_err(DatabaseError::DatabaseError)?
    {
        let manga_id = row.manga_id;
        if let Some(manga) = mangas.iter().find(|m| m.manga_id == manga_id) {
            favourites.push(Favourite {
                manga_id,
                manga: Arc::clone(manga),
                category_id: row.category_id,
                sort_key: row.sort_key,
                created_at: row.created_at,
                deleted_at: row.deleted_at,
            });
        } else {
            continue;
        }
    }

    let user = sqlx::query!(
        r#"
        SELECT
            id, favourites_sync_timestamp
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

    Ok(UserFavourite {
        favourite_categories: categories,
        favourites,
        timestamp: match user.favourites_sync_timestamp {
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

#[tracing::instrument(name = "update user_favourite", skip_all)]
pub async fn update_user_favourites(
    pool: &PgPool,
    user_id: i64,
    user_favourite: UserFavourite,
) -> Result<(), Error> {
    let mut mangas_map: HashMap<i64, Arc<Manga>> = HashMap::new();
    let mut tags_map: HashMap<i64, Arc<Tag>> = HashMap::new();
    let mut manga_tags_set = HashSet::new();

    for favourite in &user_favourite.favourites {
        for tag in &favourite.manga.tags {
            tags_map.insert(tag.tag_id, Arc::clone(tag));

            manga_tags_set.insert(MangaTagEntity {
                manga_id: favourite.manga_id,
                tag_id: tag.tag_id,
            });
        }
        mangas_map.insert(favourite.manga_id, Arc::clone(&favourite.manga));
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

    for batch in user_favourite.favourite_categories.chunks(200) {
        let mut categories_query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            INSERT INTO categories
                (id, created_at, sort_key, title, "order", user_id, track, show_in_lib, deleted_at)
        "#,
        );
        categories_query_builder.push_values(batch, |mut b, category| {
            b.push_bind(category.category_id)
                .push_bind(category.created_at)
                .push_bind(category.sort_key)
                .push_bind(&category.title)
                .push_bind(&category.order)
                .push_bind(user_id)
                .push_bind(category.track != 0)
                .push_bind(category.show_in_lib != 0)
                .push_bind(category.deleted_at);
        });
        categories_query_builder.push(
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
        categories_query_builder
            .build()
            .execute(&mut *tx)
            .await
            .map_err(DatabaseError::DatabaseError)?;
    }

    for batch in user_favourite.favourites.chunks(200) {
        let mut favourites_query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            INSERT INTO favourites
                (manga_id, category_id, user_id, sort_key, created_at, deleted_at)
        "#,
        );

        favourites_query_builder.push_values(batch, |mut b, favourite| {
            b.push_bind(favourite.manga_id)
                .push_bind(favourite.category_id)
                .push_bind(user_id)
                .push_bind(favourite.sort_key)
                .push_bind(favourite.created_at)
                .push_bind(favourite.deleted_at);
        });
        favourites_query_builder.push(
            r#"
            ON CONFLICT (manga_id, category_id, user_id)
            DO UPDATE SET 
                sort_key = EXCLUDED.sort_key,
                created_at = EXCLUDED.created_at,
                deleted_at = EXCLUDED.deleted_at;
        "#,
        );

        favourites_query_builder
            .build()
            .execute(&mut *tx)
            .await
            .map_err(DatabaseError::DatabaseError)?;
    }

    sqlx::query!(
        r#"
        UPDATE users
        SET
            favourites_sync_timestamp = $1
        WHERE 
            id = $2;
    "#,
        user_favourite.timestamp,
        // chrono::Utc::now().timestamp_millis(),
        user_id
    )
    .execute(&mut *tx)
    .await
    .map_err(DatabaseError::DatabaseError)?;

    tx.commit().await.map_err(DatabaseError::DatabaseError)?;

    drop(tags_map);
    drop(mangas_map);
    drop(manga_tags_set);

    drop(user_favourite);

    Ok(())
}

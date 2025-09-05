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

use super::{
    manga::insert_mangas, manga_tags::insert_manga_tags, tags::insert_tags,
    user::update_user_favourite_sync_time,
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

    let tags_vec: Vec<Arc<Tag>> = tags_map.values().cloned().collect();
    insert_tags(&mut tx, &tags_vec).await?;

    let mangas_vec: Vec<Arc<Manga>> = mangas_map.values().cloned().collect();
    insert_mangas(&mut tx, &mangas_vec).await?;

    let manga_tags_vec: Vec<MangaTagEntity> = manga_tags_set.into_iter().collect();
    insert_manga_tags(&mut tx, &manga_tags_vec).await?;

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

    update_user_favourite_sync_time(&mut tx, user_id).await?;

    tx.commit().await.map_err(DatabaseError::DatabaseError)?;

    drop(tags_map);
    drop(mangas_map);
    drop(manga_tags_vec);

    drop(user_favourite);

    Ok(())
}

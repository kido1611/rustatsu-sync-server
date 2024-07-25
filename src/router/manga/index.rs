use axum::debug_handler;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde_aux::field_attributes::deserialize_option_number_from_string;
use sqlx::MySqlPool;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Parameters {
    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    offset: Option<i16>,
    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    limit: Option<i16>,
}

#[derive(serde::Serialize, Clone)]
pub struct MangaEntity {
    pub id: i64,
    pub title: String,
    pub alt_title: Option<String>,
    pub url: String,
    pub public_url: String,
    pub rating: f32,
    pub is_nsfw: i8,
    pub cover_url: String,
    pub large_cover_url: Option<String>,
    pub state: Option<String>,
    pub author: Option<String>,
    pub source: String,
}

#[derive(sqlx::FromRow, serde::Serialize, PartialEq, Eq, Clone)]
pub struct TagEntity {
    pub manga_id: i64,
    pub id: i64,
    pub title: String,
    pub key: String,
    pub source: String,
}

#[derive(serde::Serialize)]
pub struct Manga {
    pub id: i64,
    pub title: String,
    pub alt_title: Option<String>,
    pub url: String,
    pub public_url: String,
    pub rating: f32,
    pub nsfw: i8,
    pub cover_url: String,
    pub large_cover_url: Option<String>,
    pub state: Option<String>,
    pub author: Option<String>,
    pub source: String,
    pub tags: Vec<Tag>,
}

#[derive(serde::Serialize)]
pub struct Tag {
    pub id: i64,
    pub title: String,
    pub key: String,
    pub source: String,
}

impl Manga {
    pub fn from_entity(entity: MangaEntity, tags: &Vec<TagEntity>) -> Self {
        let manga_tags: Vec<Tag> = tags
            .iter()
            .filter(|t| t.manga_id == entity.id)
            .map(|t| Tag {
                id: t.id,
                title: t.title.clone(),
                key: t.key.clone(),
                source: t.source.clone(),
            })
            .collect();

        Manga {
            id: entity.id,
            title: entity.title,
            alt_title: entity.alt_title,
            url: entity.url,
            public_url: entity.public_url,
            rating: entity.rating,
            nsfw: entity.is_nsfw,
            cover_url: entity.cover_url,
            large_cover_url: entity.large_cover_url,
            state: entity.state,
            author: entity.author,
            source: entity.source,
            tags: manga_tags,
        }
    }
}

#[tracing::instrument(name = "Transform manga", skip_all)]
fn transform_manga_entity_into_manga(manga: Vec<MangaEntity>, tags: &Vec<TagEntity>) -> Vec<Manga> {
    manga
        .into_iter()
        .map(|m| Manga::from_entity(m, &tags))
        .collect()
}

#[tracing::instrument(name = "Get manga", skip(pool))]
#[debug_handler]
pub async fn get_manga(
    State(pool): State<MySqlPool>,
    Query(parameters): Query<Parameters>,
) -> impl IntoResponse {
    let manga = get_manga_list(&pool, parameters).await.unwrap();
    let manga_id: Vec<i64> = manga.clone().iter().map(|m| m.id).collect();
    let tags = get_manga_tags_by_manga_id(&pool, manga_id).await.unwrap();

    Json(transform_manga_entity_into_manga(manga, &tags))
}

#[tracing::instrument(name = "Get manga list", skip(pool))]
async fn get_manga_list(
    pool: &MySqlPool,
    parameters: Parameters,
) -> Result<Vec<MangaEntity>, sqlx::Error> {
    let limit = parameters.limit.unwrap_or(20);
    let skip = parameters.offset.unwrap_or(0) * parameters.limit.unwrap_or(20);

    sqlx::query_as!(
        MangaEntity,
        r#"
               SELECT id, title, alt_title,
               url, public_url, rating, is_nsfw, cover_url, large_cover_url,
               state, author, source
               FROM manga
               ORDER BY id
               LIMIT ?
               OFFSET ?
               "#,
        limit,
        skip
    )
    .fetch_all(pool)
    .await
}

#[tracing::instrument(name = "Get manga tags by manga id", skip(pool))]
pub async fn get_manga_tags_by_manga_id(
    pool: &MySqlPool,
    manga_id: Vec<i64>,
) -> Result<Vec<TagEntity>, sqlx::Error> {
    let params = manga_id
        .iter()
        .map(|_| "?")
        .collect::<Vec<&str>>()
        .join(", ");
    let tag_sql = format!(
        "SELECT manga_tags.manga_id, tags.id, tags.title, tags.key, tags.source
                FROM tags
                JOIN manga_tags ON manga_tags.tag_id = tags.id
                WHERE manga_tags.manga_id IN ({})",
        params
    );
    let mut tag_rows = sqlx::query_as(&tag_sql);
    for mng in manga_id {
        tag_rows = tag_rows.bind(mng);
    }
    let tags: Result<Vec<TagEntity>, sqlx::Error> = tag_rows.fetch_all(pool).await;

    tags
}

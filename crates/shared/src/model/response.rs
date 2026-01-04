use std::sync::Arc;

use core_application::{
    favourite::model::FavouriteResourceOutput,
    history::model::HistoryResourceOutput,
    manga::model::{ListMangaResourceOutput, MangaResourceOutput},
    tag::model::TagDto,
};
use serde::{Deserialize, Serialize};
use tracing::{Level, instrument};

#[derive(Serialize, Deserialize)]
pub struct TagResponse {
    pub tag_id: i64,
    pub title: String,
    pub key: String,
    pub source: String,
}

impl From<TagDto> for TagResponse {
    fn from(value: TagDto) -> Self {
        TagResponse {
            tag_id: value.id,
            title: value.title,
            key: value.key,
            source: value.source,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MangaResponse {
    pub manga_id: i64,
    pub title: String,
    pub alt_title: Option<String>,
    pub url: String,
    pub public_url: String,
    pub rating: f32,
    pub nsfw: bool,
    pub content_rating: Option<String>,
    pub cover_url: String,
    pub large_cover_url: Option<String>,
    pub state: Option<String>,
    pub author: Option<String>,
    pub source: String,
    pub tags: Vec<Arc<TagResponse>>,
}

impl From<MangaResourceOutput> for MangaResponse {
    fn from(value: MangaResourceOutput) -> Self {
        let tags: Vec<Arc<TagResponse>> = value
            .tags
            .into_iter()
            .filter(|(manga_id, _)| *manga_id == value.manga.id)
            .map(|(_, tag_dto)| Arc::new(tag_dto.into()))
            .collect();

        MangaResponse {
            manga_id: value.manga.id,
            title: value.manga.title,
            alt_title: value.manga.alt_title,
            url: value.manga.url,
            public_url: value.manga.public_url,
            rating: value.manga.rating,
            content_rating: value.manga.content_rating.clone(),
            nsfw: if let Some(content_rating) = value.manga.content_rating {
                content_rating == "ADULT"
            } else {
                false
            },
            cover_url: value.manga.cover_url,
            large_cover_url: value.manga.large_cover_url,
            state: value.manga.state,
            author: value.manga.author,
            source: value.manga.source,
            tags,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct FavouriteResponse {
    pub manga_id: i64,
    pub manga: Arc<MangaResponse>,
    pub category_id: i64,
    pub sort_key: i32,
    pub created_at: i64,
    pub deleted_at: i64,
    pub pinned: bool,
}

#[derive(Serialize, Deserialize)]
pub struct CategoryResponse {
    pub category_id: i64,
    pub created_at: i64,
    pub sort_key: i32,
    pub track: bool,
    pub show_in_lib: bool,
    pub deleted_at: i64,
    pub title: String,
    pub order: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserFavouriteResponse {
    pub categories: Vec<CategoryResponse>,
    pub favourites: Vec<FavouriteResponse>,
    pub timestamp: i64,
}

impl From<FavouriteResourceOutput> for UserFavouriteResponse {
    #[instrument(name = "shared::transform_dto_to_user_favourite_response", skip_all, level = Level::DEBUG)]
    fn from(value: FavouriteResourceOutput) -> Self {
        let manga_tags_response: Vec<(i64, Arc<TagResponse>)> = value
            .manga_tags
            .into_iter()
            .map(|(manga_id, tag_dto)| (manga_id, Arc::new(tag_dto.into())))
            .collect();

        let manga_responses: Vec<Arc<MangaResponse>> = value
            .manga
            .into_iter()
            .map(|m| {
                let tags: Vec<Arc<TagResponse>> = manga_tags_response
                    .iter()
                    .filter(|(manga_id, _)| *manga_id == m.id)
                    .map(|(_, tag)| tag.clone())
                    .collect();

                Arc::new(MangaResponse {
                    manga_id: m.id,
                    title: m.title,
                    alt_title: m.alt_title,
                    url: m.url,
                    public_url: m.public_url,
                    rating: m.rating,
                    content_rating: m.content_rating.clone(),
                    nsfw: if let Some(content_rating) = m.content_rating {
                        content_rating == "ADULT"
                    } else {
                        false
                    },
                    cover_url: m.cover_url,
                    large_cover_url: m.large_cover_url,
                    state: m.state,
                    author: m.author,
                    source: m.source,
                    tags,
                })
            })
            .collect();
        let category_responses: Vec<CategoryResponse> = value
            .categories
            .into_iter()
            .map(|c| CategoryResponse {
                category_id: c.id,
                created_at: c.created_at,
                sort_key: c.sort_key,
                title: c.title,
                order: c.order,
                deleted_at: c.deleted_at,
                track: c.track,
                show_in_lib: c.show_in_lib,
            })
            .collect();

        let favourite_responses: Vec<FavouriteResponse> = value
            .favourites
            .into_iter()
            .filter(|f| manga_responses.iter().any(|m| m.manga_id == f.manga_id))
            .map(|f| {
                let manga = manga_responses
                    .iter()
                    .find(|m| m.manga_id == f.manga_id)
                    .unwrap(); // use unwrap because already checked in filter

                FavouriteResponse {
                    manga_id: f.manga_id,
                    manga: manga.clone(),
                    category_id: f.category_id,
                    sort_key: f.sort_key,
                    created_at: f.created_at,
                    deleted_at: f.deleted_at,
                    pinned: f.pinned,
                }
            })
            .collect();

        UserFavouriteResponse {
            categories: category_responses,
            favourites: favourite_responses,
            timestamp: value.sync_time,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct HistoryResponse {
    pub manga_id: i64,
    pub manga: Arc<MangaResponse>,
    pub created_at: i64,
    pub updated_at: i64,
    pub chapter_id: i64,
    pub page: i16,
    pub scroll: f32,
    pub percent: f32,
    pub chapters: i32,
    pub deleted_at: i64,
}

#[derive(Serialize, Deserialize)]
pub struct UserHistoryResponse {
    pub history: Vec<HistoryResponse>,
    pub timestamp: i64,
}

impl From<HistoryResourceOutput> for UserHistoryResponse {
    #[instrument(name = "shared::transform_dto_to_user_history_response", skip_all, level = Level::DEBUG)]
    fn from(value: HistoryResourceOutput) -> Self {
        let manga_tags_response: Vec<(i64, Arc<TagResponse>)> = value
            .manga_tags
            .into_iter()
            .map(|(manga_id, tag_dto)| (manga_id, Arc::new(tag_dto.into())))
            .collect();

        let manga_responses: Vec<Arc<MangaResponse>> = value
            .manga
            .into_iter()
            .map(|m| {
                let tags: Vec<Arc<TagResponse>> = manga_tags_response
                    .iter()
                    .filter(|(manga_id, _)| *manga_id == m.id)
                    .map(|(_, tag)| tag.clone())
                    .collect();

                Arc::new(MangaResponse {
                    manga_id: m.id,
                    title: m.title,
                    alt_title: m.alt_title,
                    url: m.url,
                    public_url: m.public_url,
                    rating: m.rating,
                    content_rating: m.content_rating.clone(),
                    nsfw: if let Some(content_rating) = m.content_rating {
                        content_rating == "ADULT"
                    } else {
                        false
                    },
                    cover_url: m.cover_url,
                    large_cover_url: m.large_cover_url,
                    state: m.state,
                    author: m.author,
                    source: m.source,
                    tags,
                })
            })
            .collect();

        let history_responses: Vec<HistoryResponse> = value
            .history
            .into_iter()
            .filter(|f| manga_responses.iter().any(|m| m.manga_id == f.manga_id))
            .map(|history_dto| {
                let manga = manga_responses
                    .iter()
                    .find(|m| m.manga_id == history_dto.manga_id)
                    .unwrap(); // use unwrap because already checked in filter

                HistoryResponse {
                    manga_id: history_dto.manga_id,
                    manga: manga.clone(),
                    created_at: history_dto.created_at,
                    updated_at: history_dto.updated_at,
                    chapter_id: history_dto.chapter_id,
                    page: history_dto.page,
                    scroll: history_dto.scroll,
                    percent: history_dto.percent,
                    chapters: history_dto.chapters,
                    deleted_at: history_dto.deleted_at,
                }
            })
            .collect();

        UserHistoryResponse {
            history: history_responses,
            timestamp: value.sync_time,
        }
    }
}

#[instrument(name = "shared::transform_dto_to_manga_list_response", skip_all, level = Level::DEBUG)]
pub fn list_manga_resource_to_list_manga_response(
    list_manga_resource: ListMangaResourceOutput,
) -> Vec<MangaResponse> {
    let manga_tags_response: Vec<(i64, Arc<TagResponse>)> = list_manga_resource
        .manga_tags
        .into_iter()
        .map(|(manga_id, tag_dto)| (manga_id, Arc::new(tag_dto.into())))
        .collect();

    let manga_responses: Vec<MangaResponse> = list_manga_resource
        .manga
        .into_iter()
        .map(|m| {
            let tags: Vec<Arc<TagResponse>> = manga_tags_response
                .iter()
                .filter(|(manga_id, _)| *manga_id == m.id)
                .map(|(_, tag)| tag.clone())
                .collect();

            MangaResponse {
                manga_id: m.id,
                title: m.title,
                alt_title: m.alt_title,
                url: m.url,
                public_url: m.public_url,
                rating: m.rating,
                content_rating: m.content_rating.clone(),
                nsfw: if let Some(content_rating) = m.content_rating {
                    content_rating == "ADULT"
                } else {
                    false
                },
                cover_url: m.cover_url,
                large_cover_url: m.large_cover_url,
                state: m.state,
                author: m.author,
                source: m.source,
                tags,
            }
        })
        .collect();

    manga_responses
}

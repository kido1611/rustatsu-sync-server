use std::collections::HashSet;

use core_application::{
    category::model::CategoryDto,
    favourite::model::{FavouriteDto, FavouriteResourceInput},
    history::model::{HistoryDto, HistoryResourceInput},
    manga::model::MangaDto,
    manga_tag::model::MangaTagDto,
    tag::model::TagDto,
};
use serde::{Deserialize, Serialize};
use tracing::{Level, instrument};

#[derive(Deserialize, Serialize)]
pub struct TagRequest {
    pub tag_id: i64,
    pub title: String,
    pub key: String,
    pub source: String,
}

#[derive(serde::Deserialize, Serialize)]
pub struct MangaRequest {
    pub manga_id: i64,
    pub title: String,
    pub alt_title: Option<String>,
    pub url: String,
    pub public_url: String,
    pub rating: f32,
    pub nsfw: Option<bool>,
    pub content_rating: Option<String>,
    pub cover_url: String,
    pub large_cover_url: Option<String>,
    pub state: Option<String>,
    pub author: Option<String>,
    pub source: String,
    pub tags: Vec<TagRequest>,
}

#[derive(serde::Deserialize, Serialize)]
pub struct FavouriteRequest {
    pub manga_id: i64,
    pub manga: MangaRequest,
    pub category_id: i64,
    pub sort_key: i32,
    pub created_at: i64,
    pub deleted_at: i64,
    pub pinned: Option<bool>,
}

#[derive(serde::Deserialize, Serialize)]
pub struct CategoryRequest {
    pub category_id: i64,
    pub created_at: i64,
    pub sort_key: i32,
    pub track: bool,
    pub show_in_lib: bool,
    pub deleted_at: i64,
    pub title: String,
    pub order: String,
}

#[derive(serde::Deserialize, Serialize)]
pub struct UserFavouriteRequest {
    pub categories: Vec<CategoryRequest>,
    pub favourites: Vec<FavouriteRequest>,
    pub timestamp: i64,
}

impl From<UserFavouriteRequest> for FavouriteResourceInput {
    #[instrument(name = "shared::transform_user_favourite_request_to_dto", skip_all, level = Level::DEBUG)]
    fn from(value: UserFavouriteRequest) -> Self {
        let mut manga_seen = HashSet::new();
        let manga: Vec<MangaDto> = value
            .favourites
            .iter()
            .filter(|item| manga_seen.insert(item.manga_id))
            .map(|item| MangaDto {
                id: item.manga.manga_id,
                title: item.manga.title.clone(),
                alt_title: item.manga.alt_title.clone(),
                url: item.manga.url.clone(),
                public_url: item.manga.public_url.clone(),
                rating: item.manga.rating,
                nsfw: item.manga.nsfw.unwrap_or_default(),
                content_rating: if item.manga.content_rating.is_some() {
                    item.manga.content_rating.clone()
                } else if let Some(nsfw) = item.manga.nsfw {
                    if nsfw {
                        Some("ADULT".to_string())
                    } else {
                        None
                    }
                } else {
                    None
                },
                cover_url: item.manga.cover_url.clone(),
                large_cover_url: item.manga.large_cover_url.clone(),
                state: item.manga.state.clone(),
                author: item.manga.author.clone(),
                source: item.manga.source.clone(),
            })
            .collect();

        let mut tags_seen = HashSet::new();
        let tags: Vec<TagDto> = value
            .favourites
            .iter()
            .flat_map(|item| {
                item.manga
                    .tags
                    .iter()
                    .filter(|item| tags_seen.insert(item.tag_id))
                    .map(|item_tag| TagDto {
                        id: item_tag.tag_id,
                        title: item_tag.title.clone(),
                        key: item_tag.key.clone(),
                        source: item_tag.source.clone(),
                    })
                    .collect::<Vec<TagDto>>()
            })
            .collect();

        let mut manga_tags_seen = HashSet::new();
        let manga_tags: Vec<MangaTagDto> = value
            .favourites
            .iter()
            .flat_map(|item| {
                item.manga
                    .tags
                    .iter()
                    .filter(|item_tag| {
                        manga_tags_seen.insert(format!("{}-{}", item.manga_id, item_tag.tag_id))
                    })
                    .map(|item_tag| MangaTagDto {
                        manga_id: item.manga_id,
                        tag_id: item_tag.tag_id,
                    })
                    .collect::<Vec<MangaTagDto>>()
            })
            .collect();

        let categories: Vec<CategoryDto> = value
            .categories
            .iter()
            .map(|item| CategoryDto {
                id: item.category_id,
                created_at: item.created_at,
                sort_key: item.sort_key,
                title: item.title.clone(),
                order: item.order.clone(),
                deleted_at: item.deleted_at,
                track: item.track,
                show_in_lib: item.show_in_lib,
            })
            .collect();

        let favourites: Vec<FavouriteDto> = value
            .favourites
            .into_iter()
            .map(|item| FavouriteDto {
                manga_id: item.manga_id,
                category_id: item.category_id,
                sort_key: item.sort_key,
                created_at: item.created_at,
                deleted_at: item.deleted_at,
                pinned: item.pinned.unwrap_or(false),
            })
            .collect();

        FavouriteResourceInput {
            manga,
            tags,
            manga_tags,
            categories,
            favourites,
            sync_time: value.timestamp,
        }
    }
}

#[derive(serde::Deserialize, Serialize)]
pub struct HistoryRequest {
    pub manga_id: i64,
    pub manga: MangaRequest,
    pub created_at: i64,
    pub updated_at: i64,
    pub chapter_id: i64,
    pub page: i16,
    pub scroll: f32,
    pub percent: f32,
    pub chapters: i32,
    pub deleted_at: i64,
}

#[derive(serde::Deserialize, Serialize)]
pub struct UserHistoryRequest {
    pub history: Vec<HistoryRequest>,
    pub timestamp: i64,
}

impl From<UserHistoryRequest> for HistoryResourceInput {
    #[instrument(name = "shared::transform_user_history_request_to_dto", skip_all, level = Level::DEBUG)]
    fn from(value: UserHistoryRequest) -> Self {
        let mut manga_seen = HashSet::new();
        let manga: Vec<MangaDto> = value
            .history
            .iter()
            .filter(|item| manga_seen.insert(item.manga_id))
            .map(|item| MangaDto {
                id: item.manga.manga_id,
                title: item.manga.title.clone(),
                alt_title: item.manga.alt_title.clone(),
                url: item.manga.url.clone(),
                public_url: item.manga.public_url.clone(),
                rating: item.manga.rating,
                nsfw: item.manga.nsfw.unwrap_or_default(),
                content_rating: if item.manga.content_rating.is_some() {
                    item.manga.content_rating.clone()
                } else if let Some(nsfw) = item.manga.nsfw {
                    if nsfw {
                        Some("ADULT".to_string())
                    } else {
                        None
                    }
                } else {
                    None
                },
                cover_url: item.manga.cover_url.clone(),
                large_cover_url: item.manga.large_cover_url.clone(),
                state: item.manga.state.clone(),
                author: item.manga.author.clone(),
                source: item.manga.source.clone(),
            })
            .collect();

        let mut tags_seen = HashSet::new();
        let tags: Vec<TagDto> = value
            .history
            .iter()
            .flat_map(|item| {
                item.manga
                    .tags
                    .iter()
                    .filter(|item| tags_seen.insert(item.tag_id))
                    .map(|item_tag| TagDto {
                        id: item_tag.tag_id,
                        title: item_tag.title.clone(),
                        key: item_tag.key.clone(),
                        source: item_tag.source.clone(),
                    })
                    .collect::<Vec<TagDto>>()
            })
            .collect();

        let mut manga_tags_seen = HashSet::new();
        let manga_tags: Vec<MangaTagDto> = value
            .history
            .iter()
            .flat_map(|item| {
                item.manga
                    .tags
                    .iter()
                    .filter(|item_tag| {
                        manga_tags_seen.insert(format!("{}-{}", item.manga_id, item_tag.tag_id))
                    })
                    .map(|item_tag| MangaTagDto {
                        manga_id: item.manga_id,
                        tag_id: item_tag.tag_id,
                    })
                    .collect::<Vec<MangaTagDto>>()
            })
            .collect();

        let history: Vec<HistoryDto> = value
            .history
            .into_iter()
            .map(|item| HistoryDto {
                manga_id: item.manga_id,
                created_at: item.created_at,
                updated_at: item.updated_at,
                chapter_id: item.chapter_id,
                page: item.page,
                scroll: item.scroll,
                percent: item.percent,
                chapters: item.chapters,
                deleted_at: item.deleted_at,
            })
            .collect();

        HistoryResourceInput {
            manga,
            tags,
            manga_tags,
            history,
            sync_time: value.timestamp,
        }
    }
}

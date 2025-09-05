use std::sync::Arc;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub nickname: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Tag {
    pub tag_id: i64,
    pub title: String,
    pub key: String,
    pub source: String,
}

#[derive(Debug)]
pub struct MangaTag {
    pub manga_id: i64,
    pub tag: Arc<Tag>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct MangaTagEntity {
    pub manga_id: i64,
    pub tag_id: i64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Manga {
    pub manga_id: i64,
    pub title: String,
    pub alt_title: Option<String>,
    pub url: String,
    pub public_url: String,
    pub rating: f32,
    pub nsfw: Option<u8>,
    pub content_rating: Option<String>,
    pub cover_url: String,
    pub large_cover_url: Option<String>,
    pub state: Option<String>,
    pub author: Option<String>,
    pub source: String,
    pub tags: Vec<Arc<Tag>>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Favourite {
    pub manga_id: i64,
    pub manga: Arc<Manga>,
    pub category_id: i64,
    pub sort_key: i32,
    pub created_at: i64,
    pub deleted_at: i64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Category {
    pub category_id: i64,
    pub created_at: i64,
    pub sort_key: i32,
    pub track: u8,
    pub show_in_lib: u8,
    pub deleted_at: i64,
    pub title: String,
    pub order: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct UserFavourite {
    pub favourite_categories: Vec<Category>,
    pub favourites: Vec<Favourite>,
    pub timestamp: i64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct History {
    pub manga_id: i64,
    pub manga: Arc<Manga>,
    pub created_at: i64,
    pub updated_at: i64,
    pub chapter_id: i64,
    pub page: i16,
    pub scroll: f32,
    pub percent: f32,
    pub chapters: i32,
    pub deleted_at: i64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct UserHistory {
    pub history: Vec<History>,
    pub timestamp: i64,
}

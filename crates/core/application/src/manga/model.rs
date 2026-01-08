use core_domain::manga::model::{Manga, MangaPagination};

use crate::tag::model::TagDto;

pub struct MangaDto {
    pub id: i64,
    pub title: String,
    pub alt_title: Option<String>,
    pub url: String,
    pub public_url: String,
    pub rating: f32,
    pub content_rating: Option<String>,
    pub nsfw: bool,
    pub cover_url: String,
    pub large_cover_url: Option<String>,
    pub state: Option<String>,
    pub author: Option<String>,
    pub source: String,
}

impl From<MangaDto> for Manga {
    fn from(mut value: MangaDto) -> Self {
        value.title = value.title.chars().take(250).collect();

        let alt_title = value.alt_title.map(|val| val.chars().take(120).collect());

        value.url = value.url.chars().take(250).collect();
        value.public_url = value.public_url.chars().take(250).collect();

        let author = value.author.map(|val| val.chars().take(120).collect());

        value.source = value.source.chars().take(120).collect();

        Manga {
            id: value.id,
            title: value.title,
            alt_title,
            url: value.url,
            public_url: value.public_url,
            rating: value.rating,
            nsfw: value.nsfw,
            content_rating: value.content_rating,
            cover_url: value.cover_url,
            large_cover_url: value.large_cover_url,
            state: value.state,
            author,
            source: value.source,
        }
    }
}

impl From<Manga> for MangaDto {
    fn from(value: Manga) -> Self {
        MangaDto {
            id: value.id,
            title: value.title,
            alt_title: value.alt_title,
            url: value.url,
            public_url: value.public_url,
            rating: value.rating,
            nsfw: value.nsfw,
            content_rating: value.content_rating,
            cover_url: value.cover_url,
            large_cover_url: value.large_cover_url,
            state: value.state,
            author: value.author,
            source: value.source,
        }
    }
}

pub struct MangaResourceOutput {
    pub manga: MangaDto,
    pub tags: Vec<(i64, TagDto)>,
}

pub struct ListMangaResourceOutput {
    pub manga: Vec<MangaDto>,
    pub manga_tags: Vec<(i64, TagDto)>,
}

pub struct ListMangaQuery {
    pub limit: i64,
    pub offset: i64,
}

impl From<ListMangaQuery> for MangaPagination {
    fn from(value: ListMangaQuery) -> Self {
        MangaPagination {
            limit: value.limit,
            offset: value.offset,
        }
    }
}

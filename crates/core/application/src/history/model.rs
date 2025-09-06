use core_domain::history::model::History;

use crate::{manga::model::MangaDto, manga_tag::model::MangaTagDto, tag::model::TagDto};

pub struct HistoryDto {
    pub manga_id: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub chapter_id: i64,
    pub page: i16,
    pub scroll: f32,
    pub percent: f32,
    pub chapters: i32,
    pub deleted_at: i64,
}

impl HistoryDto {
    pub fn to_history(self, user_id: i64) -> History {
        History {
            manga_id: self.manga_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
            chapter_id: self.chapter_id,
            page: self.page,
            scroll: self.scroll,
            percent: self.percent,
            chapters: self.chapters,
            deleted_at: self.deleted_at,
            user_id,
        }
    }
}

impl From<History> for HistoryDto {
    fn from(value: History) -> Self {
        HistoryDto {
            manga_id: value.manga_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            chapter_id: value.chapter_id,
            page: value.page,
            scroll: value.scroll,
            percent: value.percent,
            chapters: value.chapters,
            deleted_at: value.deleted_at,
        }
    }
}

pub struct HistoryResourceInput {
    pub manga: Vec<MangaDto>,
    pub tags: Vec<TagDto>,
    pub manga_tags: Vec<MangaTagDto>,
    pub history: Vec<HistoryDto>,
    pub sync_time: i64,
}

pub struct HistoryResourceOutput {
    pub manga: Vec<MangaDto>,
    pub manga_tags: Vec<(i64, TagDto)>,
    pub history: Vec<HistoryDto>,
    pub sync_time: i64,
}

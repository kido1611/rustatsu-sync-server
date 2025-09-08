use core_domain::favourite::model::Favourite;

use crate::{
    category::model::CategoryDto, manga::model::MangaDto, manga_tag::model::MangaTagDto,
    tag::model::TagDto,
};

pub struct FavouriteDto {
    pub manga_id: i64,
    pub category_id: i64,
    pub sort_key: i32,
    pub created_at: i64,
    pub deleted_at: i64,
    pub pinned: bool,
}

impl FavouriteDto {
    pub fn to_favourite(self, user_id: i64) -> Favourite {
        Favourite {
            manga_id: self.manga_id,
            category_id: self.category_id,
            sort_key: self.sort_key,
            created_at: self.created_at,
            deleted_at: self.deleted_at,
            pinned: self.pinned,
            user_id,
        }
    }
}

impl From<Favourite> for FavouriteDto {
    fn from(value: Favourite) -> Self {
        FavouriteDto {
            manga_id: value.manga_id,
            category_id: value.category_id,
            sort_key: value.sort_key,
            created_at: value.created_at,
            deleted_at: value.deleted_at,
            pinned: value.pinned,
        }
    }
}

pub struct FavouriteResourceInput {
    pub manga: Vec<MangaDto>,
    pub tags: Vec<TagDto>,
    pub manga_tags: Vec<MangaTagDto>,
    pub categories: Vec<CategoryDto>,
    pub favourites: Vec<FavouriteDto>,
    pub sync_time: i64,
}

pub struct FavouriteResourceOutput {
    pub manga: Vec<MangaDto>,
    pub manga_tags: Vec<(i64, TagDto)>,
    pub categories: Vec<CategoryDto>,
    pub favourites: Vec<FavouriteDto>,
    pub sync_time: i64,
}

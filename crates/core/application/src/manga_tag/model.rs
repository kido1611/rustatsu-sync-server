use core_domain::manga_tag::model::MangaTag;

pub struct MangaTagDto {
    pub manga_id: i64,
    pub tag_id: i64,
}

impl From<MangaTagDto> for MangaTag {
    fn from(value: MangaTagDto) -> Self {
        MangaTag {
            manga_id: value.manga_id,
            tag_id: value.tag_id,
        }
    }
}

use core_domain::tag::model::Tag;

pub struct TagDto {
    pub id: i64,
    pub title: String,
    pub key: String,
    pub source: String,
    pub pinned: bool,
}

impl From<TagDto> for Tag {
    fn from(value: TagDto) -> Self {
        Tag {
            id: value.id,
            title: value.title,
            key: value.key,
            source: value.source,
            pinned: value.pinned,
        }
    }
}

impl From<Tag> for TagDto {
    fn from(value: Tag) -> Self {
        TagDto {
            id: value.id,
            title: value.title,
            key: value.key,
            source: value.source,
            pinned: value.pinned,
        }
    }
}

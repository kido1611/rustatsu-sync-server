#[derive(serde::Serialize, Debug)]
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

#[derive(sqlx::FromRow, serde::Serialize, PartialEq, Eq)]
pub struct TagEntity {
    pub manga_id: i64,
    pub id: i64,
    pub title: String,
    pub key: String,
    pub source: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Manga {
    pub manga_id: i64,
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

impl PartialEq for Manga {
    fn eq(&self, other: &Self) -> bool {
        if self.manga_id != other.manga_id {
            return false;
        }

        if self.title != other.title {
            return false;
        }

        if self.alt_title != other.alt_title {
            return false;
        }

        if self.url != other.url {
            return false;
        }

        if self.public_url != other.public_url {
            return false;
        }

        if self.rating != other.rating {
            return false;
        }

        if self.nsfw != other.nsfw {
            return false;
        }

        if self.cover_url != other.cover_url {
            return false;
        }

        if self.large_cover_url != other.large_cover_url {
            return false;
        }

        if self.state != other.state {
            return false;
        }

        if self.author != other.author {
            return false;
        }

        if self.source != other.source {
            return false;
        }

        if self.tags != other.tags {
            return false;
        }

        true
    }
}

impl Eq for Manga {}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
pub struct Tag {
    pub tag_id: i64,
    pub title: String,
    pub key: String,
    pub source: String,
}

impl Manga {
    #[tracing::instrument(name = "transform manga", skip(tags, entity), fields(manga_id=entity.id))]
    pub fn from_entity(entity: MangaEntity, tags: &[TagEntity]) -> Self {
        let manga_tags: Vec<Tag> = tags
            .iter()
            .filter(|t| t.manga_id == entity.id)
            .map(|t| Tag {
                tag_id: t.id,
                title: t.title.to_owned(),
                key: t.key.to_owned(),
                source: t.source.to_owned(),
            })
            .collect();

        Manga {
            manga_id: entity.id,
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

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct FavouritePackage {
    pub favourite_categories: Vec<Category>,
    pub favourites: Vec<Favourite>,
    pub timestamp: i64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub struct Category {
    pub category_id: i64,
    pub created_at: i64,
    pub sort_key: i32,
    pub track: i8,
    pub title: String,
    pub order: String,
    pub deleted_at: i64,
    pub show_in_lib: i8,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Favourite {
    pub manga_id: i64,
    pub manga: Manga,
    pub category_id: i64,
    pub sort_key: i32,
    pub created_at: i64,
    pub deleted_at: i64,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct HistoryPackage {
    pub history: Vec<History>,
    pub timestamp: i64,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct History {
    pub manga_id: i64,
    pub manga: Manga,
    pub created_at: i64,
    pub updated_at: i64,
    pub chapter_id: i64,
    pub page: i16,
    pub scroll: f64,
    pub percent: f64,
    pub chapters: i32,
    pub deleted_at: i64,
}

impl PartialEq for History {
    fn eq(&self, other: &Self) -> bool {
        if self.manga_id != other.manga_id {
            return false;
        }

        if self.created_at != other.created_at {
            return false;
        }

        if self.updated_at != other.updated_at {
            return false;
        }

        if self.chapter_id != other.chapter_id {
            return false;
        }

        if self.page != other.page {
            return false;
        }

        if self.scroll != other.scroll {
            return false;
        }

        if self.percent != other.percent {
            return false;
        }

        if self.chapters != other.chapters {
            return false;
        }

        if self.deleted_at != other.deleted_at {
            return false;
        }

        true
    }
}

impl Eq for History {}

pub fn transform_manga_entity_into_manga(
    manga: Vec<MangaEntity>,
    tags: &Vec<TagEntity>,
) -> Vec<Manga> {
    manga
        .into_iter()
        .map(|m| Manga::from_entity(m, &tags))
        .collect()
}

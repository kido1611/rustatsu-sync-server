pub struct Manga {
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

pub struct MangaPagination {
    pub limit: i64,
    pub offset: i64,
}

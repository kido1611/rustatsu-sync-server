use core_application::{
    history::model::HistoryResourceInput, manga::model::MangaDto, manga_tag::model::MangaTagDto,
    tag::model::TagDto,
};
use fake::{Fake, faker::name::en::Name};
use rand::Rng;
use rustatsu_sync::state::AppState;

#[allow(dead_code)]
pub async fn insert_fake_manga(app_state: &AppState, tags_size: Option<u64>) -> i64 {
    let tags_size = tags_size.unwrap_or(2);

    let manga_dto = {
        let mut rng = rand::rng();
        let manga_id: i64 = rng.random();

        MangaDto {
            id: manga_id,
            title: Name().fake(),
            alt_title: None,
            url: "https://google.com".to_string(),
            public_url: "https://google.com/public-url".to_string(),
            rating: 5.0,
            nsfw: false,
            content_rating: None,
            cover_url: "https://google.com/cover-url".to_string(),
            large_cover_url: None,
            state: None,
            author: None,
            source: "source".to_string(),
        }
    };
    let manga_id = manga_dto.id;

    let mut tags_dto: Vec<TagDto> = Vec::new();
    let mut manga_tags_dto: Vec<MangaTagDto> = Vec::new();
    for _ in 0..tags_size {
        let tag = {
            let mut rng = rand::rng();
            let key_random: i32 = rng.random();

            TagDto {
                id: rng.random(),
                title: format!("tag title {}", Name().fake::<String>()),
                key: format!("key-{}", key_random),
                source: "source".to_string(),
                pinned: false,
            }
        };
        manga_tags_dto.push(MangaTagDto {
            manga_id: manga_dto.id,
            tag_id: tag.id,
        });
        tags_dto.push(tag);
    }

    app_state
        .insert_user_history_resource_usecase
        .execute(
            0,
            HistoryResourceInput {
                manga: vec![manga_dto],
                tags: tags_dto,
                manga_tags: manga_tags_dto,
                history: Vec::new(),
                sync_time: 0,
            },
        )
        .await
        .expect("failed when inserting fake manga");

    manga_id
}

use std::sync::Arc;

use fake::{Fake, faker::name::en::Name};
use rand::Rng;
use rustatsu_sync::{
    db::{manga::insert_mangas, manga_tags::insert_manga_tags, tags::insert_tags},
    model::{Manga, MangaTagEntity, Tag},
};
use sqlx::PgPool;

pub async fn insert_fake_manga(pool: &PgPool, tags_size: Option<u64>) -> i64 {
    let mut tx = pool.begin().await.unwrap();

    let tags_size = tags_size.unwrap_or(2);
    let mut tags = Vec::new();
    for _ in 0..tags_size {
        tags.push(Arc::new(create_fake_tag()));
    }
    insert_tags(&mut tx, &tags).await.unwrap();

    let (manga, manga_tags) = create_fake_manga(tags);
    let manga_id = manga.manga_id;

    insert_mangas(&mut tx, &[Arc::new(manga)]).await.unwrap();
    insert_manga_tags(&mut tx, &manga_tags).await.unwrap();

    tx.commit().await.unwrap();

    manga_id
}

fn create_fake_manga(tags: Vec<Arc<Tag>>) -> (Manga, Vec<MangaTagEntity>) {
    let mut manga_tags = Vec::new();

    let mut rng = rand::rng();
    let manga_id: i64 = rng.random();

    for tag in &tags {
        manga_tags.push(MangaTagEntity {
            manga_id,
            tag_id: tag.tag_id,
        });
    }

    let manga = Manga {
        manga_id,
        title: Name().fake(),
        alt_title: None,
        url: "https://google.com".to_string(),
        public_url: "https://google.com/public-url".to_string(),
        rating: 5.0,
        nsfw: Some(0),
        content_rating: None,
        cover_url: "https://google.com/cover-url".to_string(),
        large_cover_url: None,
        state: None,
        author: None,
        source: "source".to_string(),
        tags,
    };

    (manga, manga_tags)
}

fn create_fake_tag() -> Tag {
    let mut rng = rand::rng();
    let key_random: i32 = rng.random();

    Tag {
        tag_id: rng.random(),
        title: format!("tag title {}", Name().fake::<String>()),
        key: format!("key-{}", key_random),
        source: "source".to_string(),
    }
}

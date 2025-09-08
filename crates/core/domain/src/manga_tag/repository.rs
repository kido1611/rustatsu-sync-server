use async_trait::async_trait;

use crate::{manga_tag::model::MangaTag, shared::error::DomainError};

#[async_trait]
pub trait MangaTagRepository: Send + Sync {
    async fn insert(&self, manga_tags: Vec<MangaTag>) -> Result<(), DomainError>;
}

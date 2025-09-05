use async_trait::async_trait;

use crate::{shared::error::DomainError, tag::model::Tag};

#[async_trait]
pub trait TagRepository: Send + Sync {
    async fn insert(&self, tags: Vec<Tag>) -> Result<(), DomainError>;
    async fn list_by_manga_ids(&self, manga_ids: &[i64]) -> Result<Vec<(i64, Tag)>, DomainError>;
}

use async_trait::async_trait;

use crate::{
    manga::model::{Manga, MangaPagination},
    shared::error::DomainError,
};

#[async_trait]
pub trait MangaRepository: Send + Sync {
    async fn insert(&self, manga: Vec<Manga>) -> Result<(), DomainError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Manga>, DomainError>;
    async fn list_by_ids(&self, ids: &[i64]) -> Result<Vec<Manga>, DomainError>;
    async fn list_with_pagination(
        &self,
        pagination: MangaPagination,
    ) -> Result<Vec<Manga>, DomainError>;
}

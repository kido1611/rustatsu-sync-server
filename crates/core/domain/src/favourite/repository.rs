use async_trait::async_trait;

use crate::{favourite::model::Favourite, shared::error::DomainError};

#[async_trait]
pub trait FavouriteRepository: Send + Sync {
    async fn insert(&self, favourites: Vec<Favourite>) -> Result<(), DomainError>;
    async fn list_by_user_id(&self, user_id: i64) -> Result<Vec<Favourite>, DomainError>;
}

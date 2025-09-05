use async_trait::async_trait;

use crate::{category::model::Category, shared::error::DomainError};

#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn insert(&self, categories: Vec<Category>) -> Result<(), DomainError>;
    async fn list_by_user_id(&self, user_id: i64) -> Result<Vec<Category>, DomainError>;
}

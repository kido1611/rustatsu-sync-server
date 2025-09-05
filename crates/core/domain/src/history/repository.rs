use async_trait::async_trait;

use crate::{history::model::History, shared::error::DomainError};

#[async_trait]
pub trait HistoryRepository: Send + Sync {
    async fn insert(&self, history: Vec<History>) -> Result<(), DomainError>;
    async fn list_by_user_id(&self, user_id: i64) -> Result<Vec<History>, DomainError>;
}

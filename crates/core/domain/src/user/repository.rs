use async_trait::async_trait;

use crate::{
    shared::error::DomainError,
    user::model::{User, UserCreate, UserUpdateSyncTime},
};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_by_email(&self, email: &str) -> Result<Option<User>, DomainError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<User>, DomainError>;
    async fn create(&self, data: UserCreate) -> Result<User, DomainError>;
    async fn update_favourite_sync_time(&self, data: UserUpdateSyncTime)
    -> Result<(), DomainError>;
    async fn update_history_sync_time(&self, data: UserUpdateSyncTime) -> Result<(), DomainError>;
}

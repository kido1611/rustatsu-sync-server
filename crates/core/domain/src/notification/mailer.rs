use async_trait::async_trait;

use crate::{notification::model::MailEnvelope, shared::error::DomainError};

#[async_trait]
pub trait Mailer: Send + Sync {
    async fn send(&self, envelope: MailEnvelope) -> Result<(), DomainError>;
}

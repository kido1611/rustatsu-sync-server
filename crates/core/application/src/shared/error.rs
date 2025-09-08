use core_domain::shared::error::DomainError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("domain error: {0}")]
    DomainError(
        #[source]
        #[from]
        DomainError,
    ),

    #[error("user register is forbidden")]
    RegisterIsForbidden,

    #[error("resource is missing: {0}")]
    ResourceNotFound(String),
}

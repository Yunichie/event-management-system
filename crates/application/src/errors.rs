use domain::shared::errors::{DomainError, RepoError};

#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),
    #[error("Repository error: {0}")]
    Repository(#[from] RepoError),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Payment failed: {0}")]
    PaymentFailed(String),
    #[error("Refund processing failed: {0}")]
    RefundFailed(String),
    #[error("Notification failed: {0}")]
    NotificationFailed(String),
}

pub type AppResult<T> = Result<T, ApplicationError>;

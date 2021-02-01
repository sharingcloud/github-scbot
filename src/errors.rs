//! Global errors.

use thiserror::Error;

/// Bot error.
#[derive(Debug, Error)]
pub enum BotError {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Wraps [`std::io::Error`].
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    /// Wraps [`crate::api::APIError`].
    #[error(transparent)]
    APIError(#[from] crate::api::APIError),
    /// Wraps [`crate::database::DatabaseError`].
    #[error(transparent)]
    DatabaseError(#[from] crate::database::DatabaseError),
    /// Wraps [`crate::webhook::WebhookError`].
    #[error(transparent)]
    WebhookError(#[from] crate::webhook::WebhookError),
    /// Wraps [`crate::logic::LogicError`].
    #[error(transparent)]
    LogicError(#[from] crate::logic::LogicError),
}

/// Result alias for `BotError`.
pub type Result<T, E = BotError> = core::result::Result<T, E>;

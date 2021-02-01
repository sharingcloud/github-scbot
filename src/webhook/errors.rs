//! Webhook errors.

use thiserror::Error;

use crate::types::events::EventType;

/// Webhook error.
#[derive(Debug, Error)]
pub enum WebhookError {
    /// Event parsing error.
    #[error("Error while parsing webhook event {0:?}: {1}")]
    EventParseError(EventType, serde_json::Error),

    /// Wraps [`regex::Error`].
    #[error(transparent)]
    RegexError(#[from] regex::Error),

    /// Wraps [`crate::api::APIError`].
    #[error(transparent)]
    APIError(#[from] crate::api::APIError),

    /// Wraps [`crate::database::DatabaseError`].
    #[error(transparent)]
    DatabaseError(#[from] crate::database::DatabaseError),

    /// Wraps [`crate::logic::LogicError`].
    #[error(transparent)]
    LogicError(#[from] crate::logic::LogicError),
}

impl actix_web::ResponseError for WebhookError {}

/// Result alias for `WebhookError`.
pub type Result<T> = core::result::Result<T, WebhookError>;

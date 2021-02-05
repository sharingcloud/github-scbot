//! Webhook errors.

use github_scbot_types::events::EventType;
use thiserror::Error;

/// Webhook error.
#[derive(Debug, Error)]
pub enum ServerError {
    /// Event parsing error.
    #[error("Error while parsing webhook event {0:?}: {1}")]
    EventParseError(EventType, serde_json::Error),

    /// Wraps [`std::io::Error`].
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    /// Wraps [`regex::Error`].
    #[error(transparent)]
    RegexError(#[from] regex::Error),

    /// Wraps [`github_scbot_database::DatabaseError`].
    #[error(transparent)]
    DatabaseError(#[from] github_scbot_database::DatabaseError),

    /// Wraps [`github_scbot_logic::LogicError`].
    #[error(transparent)]
    LogicError(#[from] github_scbot_logic::LogicError),
}

impl actix_web::ResponseError for ServerError {}

/// Result alias for `ServerError`.
pub type Result<T> = core::result::Result<T, ServerError>;

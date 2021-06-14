//! Webhook errors.

use std::fmt;

use actix_http::error::BlockingError;
use github_scbot_types::events::EventType;
use thiserror::Error;

/// Webhook error.
#[derive(Debug, Error)]
pub enum ServerError {
    /// Event parsing error.
    #[error("Error while parsing webhook event {0:?}: {1}")]
    EventParseError(EventType, serde_json::Error),

    /// Missing signature for webhook.
    #[error("Missing webhook signature.")]
    MissingWebhookSignature,

    /// Invalid signature for webhook.
    #[error("Invalid webhook signature.")]
    InvalidWebhookSignature,

    /// Wraps [`std::io::Error`].
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// Wraps [`regex::Error`].
    #[error("Error while compiling regex.")]
    RegexError(#[from] regex::Error),

    /// Wraps [`github_scbot_database::DatabaseError`].
    #[error(transparent)]
    DatabaseError(#[from] github_scbot_database::DatabaseError),

    /// Wraps [`github_scbot_logic::LogicError`].
    #[error(transparent)]
    LogicError(#[from] github_scbot_logic::LogicError),

    /// Wraps [`github_scbot_api::ApiError`].
    #[error(transparent)]
    ApiError(#[from] github_scbot_api::ApiError),

    /// Wraps [`serde_json::Error`].
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),

    /// Threadpool error.
    #[error("Threadpool error.")]
    ThreadpoolError,
}

impl<E: Into<ServerError> + fmt::Debug + Sync + 'static> From<BlockingError<E>> for ServerError {
    fn from(err: BlockingError<E>) -> Self {
        match err {
            BlockingError::Canceled => Self::ThreadpoolError,
            BlockingError::Error(e) => e.into(),
        }
    }
}

/// Result alias for `ServerError`.
pub type Result<T> = core::result::Result<T, ServerError>;

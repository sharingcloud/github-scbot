//! Webhook errors.

use actix_http::StatusCode;
use github_scbot_sentry::WrapEyre;
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
    #[error("I/O error.")]
    IoError(#[from] std::io::Error),

    /// Wraps [`regex::Error`].
    #[error("Error while compiling regex.")]
    RegexError(#[from] regex::Error),

    /// Wraps [`github_scbot_database2::DatabaseError`].
    #[error("Database error.")]
    DatabaseError(#[from] github_scbot_database2::DatabaseError),

    /// Wraps [`github_scbot_logic::LogicError`].
    #[error("Logic error.")]
    LogicError(#[from] github_scbot_logic::LogicError),

    /// Wraps [`github_scbot_ghapi::ApiError`].
    #[error("API error.")]
    ApiError(#[from] github_scbot_ghapi::ApiError),

    /// Wraps [`serde_json::Error`].
    #[error("Serde error.")]
    SerdeError(#[from] serde_json::Error),

    /// Threadpool error.
    #[error("Threadpool error.")]
    ThreadpoolError,
}

impl From<ServerError> for WrapEyre {
    fn from(e: ServerError) -> Self {
        let status_code = match &e {
            ServerError::InvalidWebhookSignature => StatusCode::FORBIDDEN,
            ServerError::MissingWebhookSignature => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        Self::new(e.into(), status_code)
    }
}

/// Result alias for `ServerError`.
pub type Result<T> = core::result::Result<T, ServerError>;

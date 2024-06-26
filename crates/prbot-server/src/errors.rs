//! Webhook errors.

use actix_http::StatusCode;
use actix_web::ResponseError;
use thiserror::Error;

use crate::event_type::EventType;

/// Webhook error.
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Error while parsing webhook event for type {event_type}: {source}")]
    EventParseError {
        event_type: EventType,
        source: serde_json::Error,
    },

    #[error("Missing webhook signature.")]
    MissingWebhookSignature,

    #[error("Invalid webhook signature.")]
    InvalidWebhookSignature,

    #[error("I/O error: {source}")]
    IoError { source: std::io::Error },

    #[error(transparent)]
    DomainError { source: prbot_core::DomainError },

    #[error("Internal error.")]
    InternalError,
}

impl ResponseError for ServerError {
    fn status_code(&self) -> StatusCode {
        match &self {
            ServerError::InvalidWebhookSignature { .. } => StatusCode::FORBIDDEN,
            ServerError::MissingWebhookSignature { .. } => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Result alias for `ServerError`.
pub type Result<T> = core::result::Result<T, ServerError>;

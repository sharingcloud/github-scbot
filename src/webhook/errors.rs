//! Webhook errors

use thiserror::Error;

use super::types::EventType;

#[derive(Debug, Error)]
pub enum WebhookError {
    #[error("Error while parsing webhook event {0:?}: {1}")]
    EventParseError(EventType, serde_json::Error),
    #[error(transparent)]
    RegexError(#[from] regex::Error),
    #[error(transparent)]
    APIError(#[from] crate::api::errors::APIError),
    #[error(transparent)]
    DatabaseError(#[from] crate::database::errors::DatabaseError),
}

impl actix_web::ResponseError for WebhookError {}

pub type Result<T> = core::result::Result<T, WebhookError>;

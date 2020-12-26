//! Global errors

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BotError {
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    APIError(#[from] crate::api::errors::APIError),
    #[error(transparent)]
    DatabaseError(#[from] crate::database::errors::DatabaseError),
    #[error(transparent)]
    WebhookError(#[from] crate::webhook::errors::WebhookError),
}

pub type Result<T, E = BotError> = core::result::Result<T, E>;

//! Type errors.

use actix_web::http;
use thiserror::Error;

/// Type error.
#[derive(Debug, Error)]
pub enum TypeError {
    /// Unknown step label.
    #[error("Unknown step label: {0}")]
    UnknownStepLabelError(String),

    /// Unknown check status.
    #[error("Unknown check status: {0}")]
    UnknownCheckStatusError(String),

    /// Unknown QA status.
    #[error("Unknown QA status: {0}")]
    UnknownQAStatusError(String),

    /// Unsupported event.
    #[error("Unsupported event: {0}")]
    UnsupportedEvent(String),

    /// Wraps [`http::header::ToStrError`]
    #[error(transparent)]
    ToStrError(http::header::ToStrError),
}

/// Result alias for `TypeError`.
pub type Result<T> = core::result::Result<T, TypeError>;

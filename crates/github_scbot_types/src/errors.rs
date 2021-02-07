//! Type errors.

use thiserror::Error;

/// Type error.
#[derive(Debug, Error)]
pub enum TypeError {
    /// Unknown step label.
    #[error("Unknown step label: {0}")]
    UnknownStepLabel(String),

    /// Unknown check status.
    #[error("Unknown check status: {0}")]
    UnknownCheckStatus(String),

    /// Unknown QA status.
    #[error("Unknown QA status: {0}")]
    UnknownQAStatus(String),

    /// Unknown merge strategy.
    #[error("Unknown merge strategy: {0}")]
    UnknownMergeStrategy(String),

    /// Unsupported event.
    #[error("Unsupported event: {0}")]
    UnsupportedEvent(String),
}

/// Result alias for `TypeError`.
pub type Result<T> = core::result::Result<T, TypeError>;

//! Type errors.

use thiserror::Error;

/// Type error.
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum TypeError {
    /// Unknown step label.
    #[error("Unknown step label: {}", label)]
    UnknownStepLabel { label: String },

    /// Unknown check status.
    #[error("Unknown check status: {}", status)]
    UnknownCheckStatus { status: String },

    /// Unknown QA status.
    #[error("Unknown QA status: {}", status)]
    UnknownQaStatus { status: String },

    /// Unknown merge strategy.
    #[error("Unknown merge strategy: {}", strategy)]
    UnknownMergeStrategy {
        strategy: String,
        source: serde_plain::Error,
    },

    /// Invalid repository path.
    #[error("Invalid repository path: {}", path)]
    InvalidRepositoryPath { path: String },

    /// Unsupported event.
    #[error("Unsupported event: {}", event)]
    UnsupportedEvent { event: String },
}

/// Result alias for `TypeError`.
pub type Result<T> = core::result::Result<T, TypeError>;

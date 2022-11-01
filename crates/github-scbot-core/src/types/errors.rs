//! Type errors.

use snafu::prelude::*;

/// Type error.
#[allow(missing_docs)]
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum TypeError {
    /// Unknown step label.
    #[snafu(display("Unknown step label: {}", label))]
    UnknownStepLabel { label: String },

    /// Unknown check status.
    #[snafu(display("Unknown check status: {}", status))]
    UnknownCheckStatus { status: String },

    /// Unknown QA status.
    #[snafu(display("Unknown QA status: {}", status))]
    UnknownQaStatus { status: String },

    /// Unknown merge strategy.
    #[snafu(display("Unknown merge strategy: {}", strategy))]
    UnknownMergeStrategy {
        strategy: String,
        source: serde_plain::Error,
    },

    /// Invalid repository path.
    #[snafu(display("Invalid repository path: {}", path))]
    InvalidRepositoryPath { path: String },

    /// Unsupported event.
    #[snafu(display("Unsupported event: {}", event))]
    UnsupportedEvent { event: String },
}

/// Result alias for `TypeError`.
pub type Result<T> = core::result::Result<T, TypeError>;

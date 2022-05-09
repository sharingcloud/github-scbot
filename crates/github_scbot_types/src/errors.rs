//! Type errors.

use snafu::{prelude::*, Backtrace};

/// Type error.
#[allow(missing_docs)]
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum TypeError {
    /// Unknown step label.
    #[snafu(display("Unknown step label: {}", label))]
    UnknownStepLabel { label: String, backtrace: Backtrace },

    /// Unknown check status.
    #[snafu(display("Unknown check status: {}", status))]
    UnknownCheckStatus {
        status: String,
        backtrace: Backtrace,
    },

    /// Unknown QA status.
    #[snafu(display("Unknown QA status: {}", status))]
    UnknownQaStatus {
        status: String,
        backtrace: Backtrace,
    },

    /// Unknown merge strategy.
    #[snafu(display("Unknown merge strategy: {}", strategy))]
    UnknownMergeStrategy {
        strategy: String,
        source: serde_plain::Error,
        backtrace: Backtrace,
    },

    /// Invalid repository path.
    #[snafu(display("Invalid repository path: {}", path))]
    InvalidRepositoryPath { path: String, backtrace: Backtrace },

    /// Unsupported event.
    #[snafu(display("Unsupported event: {}", event))]
    UnsupportedEvent { event: String, backtrace: Backtrace },
}

/// Result alias for `TypeError`.
pub type Result<T> = core::result::Result<T, TypeError>;

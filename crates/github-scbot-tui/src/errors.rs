//! UI errors.

use snafu::prelude::*;

/// UI error.
#[allow(missing_docs)]
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum UiError {
    /// Wraps [`std::io::IoError`].
    #[snafu(display("I/O error,\n  caused by: {}", source))]
    Io { source: std::io::Error },

    /// Wraps [`std::sync::mpsc::RecvError`].
    #[snafu(display("Channel communication error,\n  caused by: {}", source))]
    Recv { source: std::sync::mpsc::RecvError },

    /// Wraps [`github_scbot_database::DatabaseError`].
    #[snafu(display("Database error,\n  caused by: {}", source))]
    Database {
        source: github_scbot_database::DatabaseError,
    },
}

impl From<std::io::Error> for UiError {
    fn from(e: std::io::Error) -> Self {
        Self::Io { source: e }
    }
}

impl From<std::sync::mpsc::RecvError> for UiError {
    fn from(e: std::sync::mpsc::RecvError) -> Self {
        Self::Recv { source: e }
    }
}

impl From<github_scbot_database::DatabaseError> for UiError {
    fn from(e: github_scbot_database::DatabaseError) -> Self {
        Self::Database { source: e }
    }
}

/// Result alias for `UiError`.
pub type Result<T> = core::result::Result<T, UiError>;

//! UI errors.

use thiserror::Error;

/// UI error.
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum UiError {
    /// Wraps [`std::io::IoError`].
    #[error("I/O error,\n  caused by: {}", source)]
    Io { source: std::io::Error },

    /// Wraps [`std::sync::mpsc::RecvError`].
    #[error("Channel communication error,\n  caused by: {}", source)]
    Recv { source: std::sync::mpsc::RecvError },

    /// Wraps [`prbot_database::DatabaseError`].
    #[error("Database error,\n  caused by: {}", source)]
    Database {
        source: prbot_database_interface::DatabaseError,
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

impl From<prbot_database_interface::DatabaseError> for UiError {
    fn from(e: prbot_database_interface::DatabaseError) -> Self {
        Self::Database { source: e }
    }
}

/// Result alias for `UiError`.
pub type Result<T> = core::result::Result<T, UiError>;

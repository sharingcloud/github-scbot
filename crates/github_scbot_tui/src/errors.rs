//! UI errors.

use thiserror::Error;

/// UI error.
#[derive(Debug, Error)]
pub enum UiError {
    /// Wraps [`std::io::IoError`].
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// Wraps [`std::sync::mpsc::RecvError`].
    #[error("Channel communication error.")]
    RecvError(#[from] std::sync::mpsc::RecvError),

    /// Wraps [`github_scbot_database::DatabaseError`].
    #[error(transparent)]
    DatabaseError(#[from] github_scbot_database::DatabaseError),
}

/// Result alias for `UiError`.
pub type Result<T> = core::result::Result<T, UiError>;

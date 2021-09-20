//! UI errors.

use thiserror::Error;

/// UI error.
#[derive(Debug, Error)]
pub enum UiError {
    /// Wraps [`std::io::IoError`].
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Wraps [`std::sync::mpsc::RecvError`].
    #[error("Channel communication error.")]
    Recv(#[from] std::sync::mpsc::RecvError),

    /// Wraps [`github_scbot_database::DatabaseError`].
    #[error(transparent)]
    Database(#[from] github_scbot_database::DatabaseError),
}

/// Result alias for `UiError`.
pub type Result<T> = core::result::Result<T, UiError>;

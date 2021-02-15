//! UI errors.

use thiserror::Error;

/// UI error.
#[derive(Debug, Error)]
pub enum UIError {
    /// Wraps [`std::io::IOError`].
    #[error("IO error.")]
    IOError(#[from] std::io::Error),

    /// Wraps [`std::sync::mpsc::RecvError`].
    #[error("Channel error.")]
    RecvError(#[from] std::sync::mpsc::RecvError),

    /// Wraps [`github_scbot_database::DatabaseError`].
    #[error(transparent)]
    DatabaseError(#[from] github_scbot_database::DatabaseError),
}

/// Result alias for `UIError`.
pub type Result<T> = core::result::Result<T, UIError>;

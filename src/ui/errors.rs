//! UI errors.

use thiserror::Error;

/// UI error.
#[derive(Debug, Error)]
pub enum UIError {
    /// Wraps [`std::io::IOError`].
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    /// Wraps [`std::sync::mpsc::RecvError`].
    #[error(transparent)]
    RecvError(#[from] std::sync::mpsc::RecvError),

    /// Wraps [`crate::database::DatabaseError`].
    #[error(transparent)]
    DatabaseError(#[from] crate::database::DatabaseError),
}

/// Result alias for `UIError`.
pub type Result<T> = core::result::Result<T, UIError>;

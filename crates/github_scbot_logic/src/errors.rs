//! Logic errors.

use actix_threadpool::BlockingError;
use thiserror::Error;

/// Logic error.
#[derive(Debug, Error)]
pub enum LogicError {
    /// Wraps [`regex::Error`].
    #[error("Error while compiling regex.")]
    RegexError(#[from] regex::Error),

    /// Wraps [`github_scbot_api::ApiError`].
    #[error(transparent)]
    ApiError(#[from] github_scbot_api::ApiError),

    /// Wraps [`github_scbot_database::DatabaseError`].
    #[error(transparent)]
    DatabaseError(#[from] github_scbot_database::DatabaseError),

    /// Threadpool error.
    #[error("Threadpool error.")]
    ThreadpoolError,
}

impl<E: Into<LogicError> + std::fmt::Debug + Sync + 'static> From<BlockingError<E>> for LogicError {
    fn from(err: BlockingError<E>) -> Self {
        match err {
            BlockingError::Canceled => Self::ThreadpoolError,
            BlockingError::Error(e) => e.into(),
        }
    }
}

/// Result alias for `LogicError`.
pub type Result<T> = core::result::Result<T, LogicError>;

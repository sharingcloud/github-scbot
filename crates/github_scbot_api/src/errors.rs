//! API errors.

use thiserror::Error;

/// API error.
#[derive(Debug, Error)]
pub enum APIError {
    /// Wraps [`octocrab::Error`].
    #[error(transparent)]
    OctocrabError(#[from] octocrab::Error),
}

/// Result alias for `APIError`.
pub type Result<T> = core::result::Result<T, APIError>;

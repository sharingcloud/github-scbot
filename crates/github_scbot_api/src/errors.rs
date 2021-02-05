//! API errors.

use thiserror::Error;

/// API error.
#[derive(Debug, Error)]
pub enum APIError {
    /// Missing pull request.
    #[error("Could not get pull-request #{1} from repository {0}")]
    MissingPullRequest(String, u64),

    /// JWT creation error.
    #[error("JWT creation error: {0}")]
    JWTCreationError(String),

    /// Merge error.
    #[error("Merge error: {0}")]
    MergeError(String),

    /// Wraps [`octocrab::Error`].
    #[error(transparent)]
    OctocrabError(#[from] octocrab::Error),
}

/// Result alias for `APIError`.
pub type Result<T> = core::result::Result<T, APIError>;

//! API errors.

use thiserror::Error;

/// API error.
#[derive(Debug, Error)]
pub enum APIError {
    /// Missing pull request.
    #[error("Could not get pull-request #{1} from repository {0}")]
    MissingPullRequest(String, u64),

    /// Wraps [`octocrab::Error`].
    #[error(transparent)]
    OctocrabError(#[from] octocrab::Error),
}

/// Result alias for `APIError`.
pub type Result<T> = core::result::Result<T, APIError>;

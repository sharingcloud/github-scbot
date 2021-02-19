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

    /// GitHub error.
    #[error("GitHub error: {0}")]
    GitHubError(String),

    /// Wraps [`octocrab::Error`].
    #[error("Octocrab error: {0}")]
    OctocrabError(#[from] octocrab::Error),

    /// Wraps [`reqwest::Error`].
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    /// Wraps [`github_scbot_crypto::CryptoError`].
    #[error("Crypto error: {0}")]
    CryptoError(#[from] github_scbot_crypto::CryptoError),
}

/// Result alias for `APIError`.
pub type Result<T> = core::result::Result<T, APIError>;

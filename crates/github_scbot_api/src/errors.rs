//! API errors.

use thiserror::Error;

/// API error.
#[derive(Debug, Error)]
pub enum ApiError {
    /// Missing pull request.
    #[error("Could not get pull-request #{1} from repository {0}")]
    MissingPullRequest(String, u64),

    /// Jwt creation error.
    #[error("Jwt creation error: {0}")]
    JwtCreationError(String),

    /// Merge error.
    #[error("Merge error: {0}")]
    MergeError(String),

    /// GitHub error.
    #[error("GitHub error: {0}")]
    GitHubError(String),

    /// Wraps [`octocrab::Error`].
    #[error("Error while using GitHub HTTP client.")]
    OctocrabError(#[from] octocrab::Error),

    /// Wraps [`reqwest::Error`].
    #[error("Error while using HTTP client.")]
    ReqwestError(#[from] reqwest::Error),

    /// Wraps [`github_scbot_crypto::CryptoError`].
    #[error(transparent)]
    CryptoError(#[from] github_scbot_crypto::CryptoError),
}

/// Result alias for `ApiError`.
pub type Result<T> = core::result::Result<T, ApiError>;

//! API errors.

use thiserror::Error;

/// API error.
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum ApiError {
    /// Merge error.
    #[error(
        "Could not merge pull request #{} on repository {}",
        pr_number,
        repository_path
    )]
    MergeError {
        pr_number: u64,
        repository_path: String,
    },

    /// Http error.
    #[error("HTTP error,\n  caused by: {}", source)]
    HttpError { source: reqwest::Error },

    /// Jwt error.
    #[error("JWT error,\n  caused by: {}", source)]
    JwtError {
        source: github_scbot_core::crypto::CryptoError,
    },
}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        ApiError::HttpError { source: e }
    }
}

/// Result alias for `ApiError`.
pub type Result<T> = core::result::Result<T, ApiError>;

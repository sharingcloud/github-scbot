//! API errors.

use thiserror::Error;

/// API error.
#[derive(Debug, Error, Clone)]
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

    /// HTTP error
    #[error("HTTP error: {0}")]
    HTTPError(String),

    /// JWT Error
    #[error("JWT error: {0}")]
    JWTError(String),
}

impl From<reqwest::Error> for ApiError {
    fn from(error: reqwest::Error) -> Self {
        Self::HTTPError(error.to_string())
    }
}

/// Result alias for `ApiError`.
pub type Result<T> = core::result::Result<T, ApiError>;

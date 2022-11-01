//! API errors.

use snafu::prelude::*;

/// API error.
#[allow(missing_docs)]
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum ApiError {
    /// Merge error.
    #[snafu(display(
        "Could not merge pull request #{} on repository {}",
        pr_number,
        repository_path
    ))]
    MergeError {
        pr_number: u64,
        repository_path: String,
    },

    /// Http error.
    #[snafu(display("HTTP error,\n  caused by: {}", source))]
    HttpError { source: reqwest::Error },

    /// Jwt error.
    #[snafu(display("JWT error,\n  caused by: {}", source))]
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

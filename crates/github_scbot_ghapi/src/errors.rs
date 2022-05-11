//! API errors.

use snafu::{prelude::*, Backtrace};

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
        backtrace: Backtrace,
    },

    /// Http error.
    #[snafu(display("HTTP error,\n  caused by: {}", source))]
    HttpError {
        source: reqwest::Error,
        backtrace: Backtrace,
    },

    /// Jwt error.
    #[snafu(display("JWT error,\n  caused by: {}", source))]
    JwtError {
        #[snafu(backtrace)]
        source: github_scbot_crypto::CryptoError,
    },
}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        ApiError::HttpError {
            source: e,
            backtrace: Backtrace::new(),
        }
    }
}

/// Result alias for `ApiError`.
pub type Result<T> = core::result::Result<T, ApiError>;

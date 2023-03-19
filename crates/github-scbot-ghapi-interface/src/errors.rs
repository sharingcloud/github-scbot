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

    #[error(transparent)]
    ImplementationError {
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
}

/// Result alias for `ApiError`.
pub type Result<T, E = ApiError> = core::result::Result<T, E>;

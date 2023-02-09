use thiserror::Error;

/// Lock error.
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum LockError {
    /// Implementation-specific error
    #[error(transparent)]
    ImplementationError {
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
}

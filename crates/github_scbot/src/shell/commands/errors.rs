//! Command error.

use stable_eyre::eyre;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum CommandError {
    #[error("Cannot remove default strategy.")]
    CannotRemoveDefaultStrategy,
    #[error("Sentry is not configured.")]
    SentryNotConfigured,

    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    DatabaseError(#[from] github_scbot_database::DatabaseError),
    #[error(transparent)]
    LogicError(#[from] github_scbot_logic::LogicError),
    #[error(transparent)]
    TypeError(#[from] github_scbot_types::TypeError),
    #[error(transparent)]
    ExportError(#[from] github_scbot_database::import_export::ExportError),
    #[error(transparent)]
    ImportError(#[from] github_scbot_database::import_export::ImportError),
}

pub(crate) type Result<T, E = CommandError> = eyre::Result<T, E>;

//! UI errors

use thiserror::Error;

#[derive(Debug, Error)]
pub enum UIError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    RecvError(#[from] std::sync::mpsc::RecvError),

    #[error(transparent)]
    DatabaseError(#[from] crate::database::errors::DatabaseError)
}

pub type Result<T> = core::result::Result<T, UIError>;

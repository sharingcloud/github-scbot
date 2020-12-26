//! API errors

use thiserror::Error;

#[derive(Debug, Error)]
pub enum APIError {
    #[error(transparent)]
    OctocrabError(#[from] octocrab::Error),

    #[error(transparent)]
    DatabaseError(#[from] crate::database::errors::DatabaseError),
}

pub type Result<T> = core::result::Result<T, APIError>;

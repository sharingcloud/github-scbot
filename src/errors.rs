//! Global errors

use std::env::VarError;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BotError {
    #[error("Bad check status: {0}")]
    BadCheckStatus(String),
    #[error("Bad QA status: {0}")]
    BadQAStatus(String),
    #[error("DB error: {0}")]
    DBError(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Format error: {0}")]
    FormatError(String),
    #[error("Event parse error: {0}")]
    EventParseError(String),
    #[error("Unknown label name: {0}")]
    UnknownLabelName(String),
    #[error("Octobrab error: {0}")]
    OctocrabError(#[from] octocrab::Error),
    #[error("Regex compiling error: {0}")]
    RegexError(#[from] regex::Error),
    #[error("Int conversion error: {0}")]
    TryFromIntError(#[from] std::num::TryFromIntError),
    #[error("Missing or wrong environment variable: {0}")]
    EnvVarError(#[from] VarError),
    #[error("Unknown pull request #{1} from repository {0}")]
    UnknownPullRequest(String, u64),
}

impl actix_web::ResponseError for BotError {}

pub type Result<T, E = BotError> = core::result::Result<T, E>;

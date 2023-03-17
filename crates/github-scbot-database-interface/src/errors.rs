use github_scbot_core::{crypto::CryptoError, types::rule_branch::RuleBranch};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Import/Export JSON error")]
    ExchangeJsonError { source: serde_json::Error },

    #[error("Crypto error.")]
    CryptoError { source: CryptoError },

    #[error("Unknown repository path '{0}'")]
    UnknownRepository(String),

    #[error("Unknown repository ID '{0}'")]
    UnknownRepositoryId(u64),

    #[error("Unknown account '{0}'")]
    UnknownAccount(String),

    #[error("Unknown merge rule '{0}' -> '{1}'")]
    UnknownMergeRule(RuleBranch, RuleBranch),

    #[error("Unknown external account '{0}'")]
    UnknownExternalAccount(String),

    #[error("Unknown pull request '#{1}' for repository path '{0}'")]
    UnknownPullRequest(String, u64),

    #[error("Unknown pull request ID '{0}'")]
    UnknownPullRequestId(u64),

    #[error(transparent)]
    ImplementationError {
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
}

pub type Result<T, E = DatabaseError> = core::result::Result<T, E>;

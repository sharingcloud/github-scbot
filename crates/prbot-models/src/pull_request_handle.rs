use std::str::FromStr;

use thiserror::Error;

use crate::RepositoryPath;

/// Type error.
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum PullRequestHandleError {
    /// Invalid pull request handle.
    #[error("Invalid pull request handle: {}", path)]
    InvalidPullRequestHandle { path: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullRequestHandle {
    repository: RepositoryPath,
    number: u64,
}

impl std::fmt::Display for PullRequestHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} (#{})", self.repository, self.number))
    }
}

impl PullRequestHandle {
    pub fn new(repository: RepositoryPath, number: u64) -> Self {
        Self { repository, number }
    }

    pub fn repository_path(&self) -> &RepositoryPath {
        &self.repository
    }

    pub fn number(&self) -> u64 {
        self.number
    }

    pub fn owner(&self) -> &str {
        self.repository.owner()
    }

    pub fn name(&self) -> &str {
        self.repository.name()
    }
}

impl From<(&str, &str, u64)> for PullRequestHandle {
    fn from((owner, name, number): (&str, &str, u64)) -> Self {
        Self {
            repository: (owner, name).into(),
            number,
        }
    }
}

impl FromStr for PullRequestHandle {
    type Err = PullRequestHandleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

impl TryFrom<&str> for PullRequestHandle {
    type Error = PullRequestHandleError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Some((path, number)) = value.rsplit_once('/') {
            let repo_path = RepositoryPath::try_from(path).map_err(|_| {
                PullRequestHandleError::InvalidPullRequestHandle { path: value.into() }
            })?;
            let number = number.parse::<u64>().map_err(|_| {
                PullRequestHandleError::InvalidPullRequestHandle { path: value.into() }
            })?;
            Ok(Self::new(repo_path, number))
        } else {
            Err(PullRequestHandleError::InvalidPullRequestHandle { path: value.into() })
        }
    }
}

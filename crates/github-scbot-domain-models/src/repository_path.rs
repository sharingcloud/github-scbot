//! Repository types.

use std::str::FromStr;

use thiserror::Error;

/// Type error.
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum RepositoryPathError {
    /// Invalid repository path.
    #[error("Invalid repository path: {}", path)]
    InvalidRepositoryPath { path: String },
}

/// Repository path.
#[derive(Debug, Clone)]
pub struct RepositoryPath {
    owner: String,
    name: String,
}

impl RepositoryPath {
    /// Creates a new repository path.
    pub fn new(path: &str) -> Result<Self, RepositoryPathError> {
        let (owner, name) = Self::split_repo_path(path)?;

        Ok(Self {
            owner: owner.into(),
            name: name.into(),
        })
    }

    /// Creates a new repository path from components
    pub fn new_from_components(owner: &str, name: &str) -> Self {
        Self {
            owner: owner.into(),
            name: name.into(),
        }
    }

    /// Get owner and name.
    pub fn components(&self) -> (&str, &str) {
        (&self.owner, &self.name)
    }

    /// Get owner.
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Get name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get full name.
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    fn split_repo_path(repo_path: &str) -> Result<(&str, &str), RepositoryPathError> {
        let split: Vec<_> = repo_path.split('/').collect();

        if split.len() == 2 {
            Ok((split[0], split[1]))
        } else {
            Err(RepositoryPathError::InvalidRepositoryPath {
                path: repo_path.to_string(),
            })
        }
    }
}

impl FromStr for RepositoryPath {
    type Err = RepositoryPathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

impl TryFrom<&str> for RepositoryPath {
    type Error = RepositoryPathError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl std::fmt::Display for RepositoryPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.full_name())
    }
}

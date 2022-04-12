//! Repository types.

use std::str::FromStr;

use crate::TypeError;

/// Repository path.
#[derive(Debug)]
pub struct RepositoryPath {
    owner: String,
    name: String,
}

impl RepositoryPath {
    /// Creates a new repository path.
    pub fn new(path: &str) -> Result<Self, TypeError> {
        let (owner, name) = Self::split_repo_path(path)?;

        Ok(Self {
            owner: owner.into(),
            name: name.into(),
        })
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

    fn split_repo_path(repo_path: &str) -> Result<(&str, &str), TypeError> {
        let split: Vec<_> = repo_path.split('/').collect();

        if split.len() == 2 {
            Ok((split[0], split[1]))
        } else {
            Err(TypeError::InvalidRepositoryPath(repo_path.into()))
        }
    }
}

impl FromStr for RepositoryPath {
    type Err = TypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl std::fmt::Display for RepositoryPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.full_name())
    }
}

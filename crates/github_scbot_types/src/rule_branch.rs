//! Rule branch.

use std::{convert::Infallible, str::FromStr};

use serde::{Deserialize, Serialize};

/// Rule branch.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RuleBranch {
    /// Named.
    Named(String),
    /// Wildcard.
    Wildcard,
}

impl From<&str> for RuleBranch {
    fn from(value: &str) -> Self {
        match value {
            "*" => Self::Wildcard,
            n => Self::Named(n.into()),
        }
    }
}

impl FromStr for RuleBranch {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl RuleBranch {
    /// Get branch name.
    pub fn name(&self) -> String {
        match self {
            RuleBranch::Named(s) => s.clone(),
            RuleBranch::Wildcard => "*".into(),
        }
    }
}

impl Default for RuleBranch {
    fn default() -> Self {
        RuleBranch::Wildcard
    }
}

impl std::fmt::Display for RuleBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name())
    }
}

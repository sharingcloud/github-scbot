use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MergeStrategyError {
    /// Unknown merge strategy.
    #[error("Unknown merge strategy: {}", strategy)]
    UnknownMergeStrategy { strategy: String },
}

#[derive(Debug, Serialize, Default, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MergeStrategy {
    /// Merge
    #[default]
    Merge,
    /// Squash
    Squash,
    /// Rebase
    Rebase,
}

impl std::fmt::Display for MergeStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Merge => "merge",
            Self::Squash => "squash",
            Self::Rebase => "rebase",
        };

        f.write_str(value)
    }
}

impl FromStr for MergeStrategy {
    type Err = MergeStrategyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl TryFrom<&str> for MergeStrategy {
    type Error = MergeStrategyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "merge" => Ok(Self::Merge),
            "squash" => Ok(Self::Squash),
            "rebase" => Ok(Self::Rebase),
            other => Err(MergeStrategyError::UnknownMergeStrategy {
                strategy: other.into(),
            }),
        }
    }
}

impl TryFrom<&String> for MergeStrategy {
    type Error = MergeStrategyError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

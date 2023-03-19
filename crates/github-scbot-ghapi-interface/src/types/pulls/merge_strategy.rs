use serde::{Deserialize, Serialize};

/// GitHub Merge strategy.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GhMergeStrategy {
    /// Merge
    Merge,
    /// Squash
    Squash,
    /// Rebase
    Rebase,
}

impl std::fmt::Display for GhMergeStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Merge => "merge",
            Self::Squash => "squash",
            Self::Rebase => "rebase",
        };

        f.write_str(value)
    }
}

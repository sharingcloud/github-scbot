use serde::{Deserialize, Serialize};

use super::GhUser;

/// GitHub Branch.
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct GhBranch {
    /// Label.
    pub label: Option<String>,
    #[serde(rename = "ref")]
    /// Reference.
    pub reference: String,
    /// SHA.
    pub sha: String,
    /// Creator.
    pub user: Option<GhUser>,
}

/// GitHub Branch (short format).
#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Eq, Clone)]
pub struct GhBranchShort {
    /// Reference.
    #[serde(rename = "ref")]
    pub reference: String,
    /// SHA.
    pub sha: String,
}

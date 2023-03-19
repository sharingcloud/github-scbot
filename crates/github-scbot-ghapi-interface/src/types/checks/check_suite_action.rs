use serde::{Deserialize, Serialize};

/// GitHub Check suite action.
#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GhCheckSuiteAction {
    /// Completed.
    #[default]
    Completed,
    /// Requested.
    Requested,
    /// Re-requested.
    Rerequested,
}

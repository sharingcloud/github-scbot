use serde::{Deserialize, Serialize};

/// GitHub Check status.
#[derive(Debug, Deserialize, Serialize, Copy, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GhCheckStatus {
    /// Completed.
    #[default]
    Completed,
    /// In progress.
    InProgress,
    /// Queued.
    Queued,
    /// Requested.
    Requested,
    /// Pending.
    Pending,
}

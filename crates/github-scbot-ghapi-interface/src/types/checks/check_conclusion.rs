use serde::{Deserialize, Serialize};

/// GitHub Check conclusion.
#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum GhCheckConclusion {
    /// Action required.
    ActionRequired,
    /// Cancelled.
    Cancelled,
    /// Failure.
    Failure,
    /// Neutral.
    Neutral,
    /// Skipped.
    Skipped,
    /// Stale.
    Stale,
    /// Startup failure.
    StartupFailure,
    /// Success.
    #[default]
    Success,
    /// Timed out.
    TimedOut,
}

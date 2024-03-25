use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use time::OffsetDateTime;

/// GitHub commit status
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, SmartDefault)]
pub struct GhCommitStatus {
    pub state: GhCommitStatusState,
    pub items: Vec<GhCommitStatusItem>,
}

/// GitHub commit status item
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, SmartDefault)]
pub struct GhCommitStatusItem {
    pub state: GhCommitStatusState,
    pub context: String,
    #[default(OffsetDateTime::now_utc())]
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[default(OffsetDateTime::now_utc())]
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// GitHub commit status state
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy, SmartDefault)]
pub enum GhCommitStatusState {
    /// Error.
    Error,
    /// Failure.
    Failure,
    /// Pending.
    #[default]
    Pending,
    /// Success.
    Success,
}

impl GhCommitStatusState {
    /// Convert status state to static str.
    pub fn to_str(self) -> &'static str {
        self.into()
    }
}

impl From<GhCommitStatusState> for &'static str {
    fn from(status_state: GhCommitStatusState) -> Self {
        match status_state {
            GhCommitStatusState::Error => "error",
            GhCommitStatusState::Failure => "failure",
            GhCommitStatusState::Pending => "pending",
            GhCommitStatusState::Success => "success",
        }
    }
}

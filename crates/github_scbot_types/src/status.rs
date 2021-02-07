//! Status types.

use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

use super::errors::TypeError;

/// Status state.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StatusState {
    /// Error.
    Error,
    /// Failure.
    Failure,
    /// Pending.
    Pending,
    /// Success.
    Success,
}

impl StatusState {
    /// Convert status state to static str.
    pub fn to_str(self) -> &'static str {
        self.into()
    }
}

impl From<StatusState> for &'static str {
    fn from(status_state: StatusState) -> Self {
        match status_state {
            StatusState::Error => "error",
            StatusState::Failure => "failure",
            StatusState::Pending => "pending",
            StatusState::Success => "success",
        }
    }
}

/// Check status.
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    /// Waiting.
    Waiting,
    /// Skipped.
    Skipped,
    /// Pass.
    Pass,
    /// Fail.
    Fail,
}

impl CheckStatus {
    /// Convert check status to static str.
    pub fn to_str(self) -> &'static str {
        self.into()
    }
}

impl TryFrom<&str> for CheckStatus {
    type Error = TypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pass" => Ok(Self::Pass),
            "waiting" => Ok(Self::Waiting),
            "skipped" => Ok(Self::Skipped),
            "fail" => Ok(Self::Fail),
            e => Err(TypeError::UnknownCheckStatus(e.to_string())),
        }
    }
}

impl From<CheckStatus> for &'static str {
    fn from(check_status: CheckStatus) -> Self {
        match check_status {
            CheckStatus::Waiting => "waiting",
            CheckStatus::Skipped => "skipped",
            CheckStatus::Pass => "pass",
            CheckStatus::Fail => "fail",
        }
    }
}

/// QA status.
#[derive(Debug, Deserialize, Serialize, PartialEq, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum QAStatus {
    /// Waiting.
    Waiting,
    /// Skipped.
    Skipped,
    /// Pass.
    Pass,
    /// Fail.
    Fail,
}

impl QAStatus {
    /// Convert QA status to static str.
    pub fn to_str(self) -> &'static str {
        self.into()
    }
}

impl TryFrom<&str> for QAStatus {
    type Error = TypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pass" => Ok(Self::Pass),
            "waiting" => Ok(Self::Waiting),
            "fail" => Ok(Self::Fail),
            "skipped" => Ok(Self::Skipped),
            e => Err(TypeError::UnknownQAStatus(e.to_string())),
        }
    }
}

impl From<QAStatus> for &'static str {
    fn from(qa_status: QAStatus) -> Self {
        match qa_status {
            QAStatus::Waiting => "waiting",
            QAStatus::Pass => "pass",
            QAStatus::Skipped => "skipped",
            QAStatus::Fail => "fail",
        }
    }
}

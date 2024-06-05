use std::fmt::Display;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChecksStatusError {
    /// Unknown check status.
    #[error("Unknown check status: {}", status)]
    UnknownChecksStatus { status: String },
}

/// Checks status.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChecksStatus {
    /// Waiting.
    Waiting,
    /// Skipped.
    #[default]
    Skipped,
    /// Pass.
    Pass,
    /// Fail.
    Fail,
}

impl ChecksStatus {
    /// Convert check status to static str.
    pub fn to_str(self) -> &'static str {
        self.into()
    }
}

impl Display for ChecksStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_str())
    }
}

impl TryFrom<&str> for ChecksStatus {
    type Error = ChecksStatusError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pass" => Ok(Self::Pass),
            "waiting" => Ok(Self::Waiting),
            "skipped" => Ok(Self::Skipped),
            "fail" => Ok(Self::Fail),
            e => Err(ChecksStatusError::UnknownChecksStatus {
                status: e.to_string(),
            }),
        }
    }
}

impl TryFrom<&String> for ChecksStatus {
    type Error = ChecksStatusError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

impl From<ChecksStatus> for &'static str {
    fn from(check_status: ChecksStatus) -> Self {
        match check_status {
            ChecksStatus::Waiting => "waiting",
            ChecksStatus::Skipped => "skipped",
            ChecksStatus::Pass => "pass",
            ChecksStatus::Fail => "fail",
        }
    }
}

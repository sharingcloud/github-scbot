use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChecksStatusError {
    /// Unknown check status.
    #[error("Unknown check status: {}", status)]
    UnknownChecksStatus { status: String },
}

/// Checks status.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ChecksStatus {
    /// Waiting.
    Waiting,
    /// Skipped.
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

impl ToString for ChecksStatus {
    fn to_string(&self) -> String {
        self.to_str().into()
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

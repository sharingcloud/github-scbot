use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QaStatusError {
    /// Unknown QA status.
    #[error("Unknown QA status: {}", status)]
    UnknownQaStatus { status: String },
}

/// QA status.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Copy, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum QaStatus {
    /// Waiting.
    #[default]
    Waiting,
    /// Skipped.
    Skipped,
    /// Pass.
    Pass,
    /// Fail.
    Fail,
}

impl QaStatus {
    /// Convert QA status to static str.
    pub fn to_str(self) -> &'static str {
        self.into()
    }
}

impl Display for QaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_str())
    }
}

impl FromStr for QaStatus {
    type Err = QaStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl TryFrom<&str> for QaStatus {
    type Error = QaStatusError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pass" => Ok(Self::Pass),
            "waiting" => Ok(Self::Waiting),
            "fail" => Ok(Self::Fail),
            "skipped" => Ok(Self::Skipped),
            e => Err(QaStatusError::UnknownQaStatus {
                status: e.to_string(),
            }),
        }
    }
}

impl TryFrom<&String> for QaStatus {
    type Error = QaStatusError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

impl From<QaStatus> for &'static str {
    fn from(qa_status: QaStatus) -> Self {
        match qa_status {
            QaStatus::Waiting => "waiting",
            QaStatus::Pass => "pass",
            QaStatus::Skipped => "skipped",
            QaStatus::Fail => "fail",
        }
    }
}

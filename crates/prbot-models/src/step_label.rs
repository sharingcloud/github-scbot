//! Label types.

use std::{convert::TryFrom, fmt::Display};

use thiserror::Error;

/// Type error.
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum StepLabelError {
    /// Unknown step label.
    #[error("Unknown step label: {}", label)]
    UnknownStepLabel { label: String },
}

/// Step label.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StepLabel {
    /// Work in progress.
    Wip,
    /// Awaiting changes.
    AwaitingChanges,
    /// Awaiting checks.
    AwaitingChecks,
    /// Awaiting review.
    AwaitingReview,
    /// Awaiting required review.
    AwaitingRequiredReview,
    /// Awaiting QA.
    AwaitingQa,
    /// Locked
    Locked,
    /// Awaiting merge.
    AwaitingMerge,
}

impl StepLabel {
    /// Convert step label to static str.
    pub fn to_str(self) -> &'static str {
        self.into()
    }
}

impl Display for StepLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_str())
    }
}

impl TryFrom<&str> for StepLabel {
    type Error = StepLabelError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "step/wip" => Ok(Self::Wip),
            "step/awaiting-checks" => Ok(Self::AwaitingChecks),
            "step/awaiting-changes" => Ok(Self::AwaitingChanges),
            "step/awaiting-review" => Ok(Self::AwaitingReview),
            "step/awaiting-required-review" => Ok(Self::AwaitingRequiredReview),
            "step/awaiting-qa" => Ok(Self::AwaitingQa),
            "step/awaiting-merge" => Ok(Self::AwaitingMerge),
            "step/locked" => Ok(Self::Locked),
            name => Err(StepLabelError::UnknownStepLabel {
                label: name.to_string(),
            }),
        }
    }
}

impl TryFrom<&String> for StepLabel {
    type Error = StepLabelError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

impl From<StepLabel> for &'static str {
    fn from(label: StepLabel) -> Self {
        match label {
            StepLabel::Wip => "step/wip",
            StepLabel::AwaitingChecks => "step/awaiting-checks",
            StepLabel::AwaitingChanges => "step/awaiting-changes",
            StepLabel::AwaitingReview => "step/awaiting-review",
            StepLabel::AwaitingRequiredReview => "step/awaiting-required-review",
            StepLabel::AwaitingQa => "step/awaiting-qa",
            StepLabel::AwaitingMerge => "step/awaiting-merge",
            StepLabel::Locked => "step/locked",
        }
    }
}

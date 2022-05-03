//! Label types.

use std::convert::TryFrom;

use super::errors::TypeError;

/// Step label.
#[derive(Debug, Copy, Clone, PartialEq)]
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

impl ToString for StepLabel {
    fn to_string(&self) -> String {
        self.to_str().into()
    }
}

impl TryFrom<&str> for StepLabel {
    type Error = TypeError;

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
            name => Err(TypeError::UnknownStepLabel(name.to_string())),
        }
    }
}

impl TryFrom<&String> for StepLabel {
    type Error = TypeError;

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

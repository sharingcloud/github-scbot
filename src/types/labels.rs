//! Labels types.

use std::convert::TryFrom;

use super::errors::TypeError;

/// Step label.
#[derive(Debug, Copy, Clone)]
pub enum StepLabel {
    /// Work in progress.
    Wip,
    /// Awaiting checks.
    AwaitingChecks,
    /// Awaiting checks changes.
    AwaitingChecksChanges,
    /// Awaiting review.
    AwaitingReview,
    /// Awaiting review changes.
    AwaitingReviewChanges,
    /// Awaiting QA.
    AwaitingQA,
    /// Awaiting merge.
    AwaitingMerge,
}

impl StepLabel {
    /// Convert step label to static str.
    pub fn to_str(self) -> &'static str {
        self.into()
    }
}

impl TryFrom<&str> for StepLabel {
    type Error = TypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "step/wip" => Ok(Self::Wip),
            "step/awaiting-checks" => Ok(Self::AwaitingChecks),
            "step/awaiting-checks-changes" => Ok(Self::AwaitingChecksChanges),
            "step/awaiting-review" => Ok(Self::AwaitingReview),
            "step/awaiting-review-changes" => Ok(Self::AwaitingReviewChanges),
            "step/awaiting-qa" => Ok(Self::AwaitingQA),
            "step/awaiting-merge" => Ok(Self::AwaitingMerge),
            name => Err(TypeError::UnknownStepLabelError(name.to_string())),
        }
    }
}

impl From<StepLabel> for &'static str {
    fn from(label: StepLabel) -> Self {
        match label {
            StepLabel::Wip => "step/wip",
            StepLabel::AwaitingChecks => "step/awaiting-checks",
            StepLabel::AwaitingChecksChanges => "step/awaiting-checks-changes",
            StepLabel::AwaitingReview => "step/awaiting-review",
            StepLabel::AwaitingReviewChanges => "step/awaiting-review-changes",
            StepLabel::AwaitingQA => "step/awaiting-qa",
            StepLabel::AwaitingMerge => "step/awaiting-merge",
        }
    }
}

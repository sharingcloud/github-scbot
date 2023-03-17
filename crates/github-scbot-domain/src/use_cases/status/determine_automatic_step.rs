use github_scbot_domain_models::{ChecksStatus, QaStatus, StepLabel};

use super::PullRequestStatus;

pub struct DetermineAutomaticStepUseCase<'a> {
    pub pr_status: &'a PullRequestStatus,
}

impl<'a> DetermineAutomaticStepUseCase<'a> {
    #[tracing::instrument(skip(self), ret)]
    pub fn run(&mut self) -> StepLabel {
        if self.pr_status.wip {
            StepLabel::Wip
        } else if self.pr_status.valid_pr_title {
            match self.pr_status.checks_status {
                ChecksStatus::Pass | ChecksStatus::Skipped => {
                    if self.pr_status.changes_required()
                        || !self.pr_status.mergeable && !self.pr_status.merged
                    {
                        StepLabel::AwaitingChanges
                    } else if self.pr_status.missing_required_reviews() {
                        StepLabel::AwaitingRequiredReview
                    } else if self.pr_status.missing_reviews() {
                        StepLabel::AwaitingReview
                    } else {
                        match self.pr_status.qa_status {
                            QaStatus::Fail => StepLabel::AwaitingChanges,
                            QaStatus::Waiting => StepLabel::AwaitingQa,
                            QaStatus::Pass | QaStatus::Skipped => {
                                if self.pr_status.locked {
                                    StepLabel::Locked
                                } else {
                                    StepLabel::AwaitingMerge
                                }
                            }
                        }
                    }
                }
                ChecksStatus::Waiting => StepLabel::AwaitingChecks,
                ChecksStatus::Fail => StepLabel::AwaitingChanges,
            }
        } else {
            StepLabel::AwaitingChanges
        }
    }
}
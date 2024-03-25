use prbot_models::{ChecksStatus, QaStatus, StepLabel};

use super::PullRequestStatus;

#[derive(Default)]
pub struct StepLabelChooser {
    _private: (),
}

impl StepLabelChooser {
    pub fn choose_from_status(&self, pr_status: &PullRequestStatus) -> StepLabel {
        if pr_status.wip {
            StepLabel::Wip
        } else if pr_status.valid_pr_title {
            match pr_status.checks_status {
                ChecksStatus::Pass | ChecksStatus::Skipped => {
                    if pr_status.changes_required() || !pr_status.mergeable && !pr_status.merged {
                        StepLabel::AwaitingChanges
                    } else if pr_status.missing_required_reviews() {
                        StepLabel::AwaitingRequiredReview
                    } else if pr_status.missing_reviews() {
                        StepLabel::AwaitingReview
                    } else {
                        match pr_status.qa_status {
                            QaStatus::Fail => StepLabel::AwaitingChanges,
                            QaStatus::Waiting => StepLabel::AwaitingQa,
                            QaStatus::Pass | QaStatus::Skipped => {
                                if pr_status.locked {
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

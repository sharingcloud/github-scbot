use github_scbot_core::types::status::{CheckStatus, QaStatus, StatusState};

use crate::status::PullRequestStatus;
use crate::Result;

pub const VALIDATION_STATUS_MESSAGE: &str = "Validation";

pub struct StatusMessage {
    pub state: StatusState,
    pub title: &'static str,
    pub message: String,
}

pub struct GenerateStatusMessageUseCase<'a> {
    pub pr_status: &'a PullRequestStatus,
}

impl<'a> GenerateStatusMessageUseCase<'a> {
    pub fn run(&mut self) -> Result<StatusMessage> {
        let status_title = VALIDATION_STATUS_MESSAGE;
        let mut status_state = StatusState::Success;
        let mut status_message = "All good.".to_string();

        if self.pr_status.wip {
            status_message = "PR is still in WIP".to_string();
            status_state = StatusState::Failure;
        } else if self.pr_status.valid_pr_title {
            // Check CI status
            match self.pr_status.checks_status {
                CheckStatus::Fail => {
                    status_message = "Checks failed. Please fix.".to_string();
                    status_state = StatusState::Failure;
                }
                CheckStatus::Waiting => {
                    status_message = "Waiting for checks".to_string();
                    status_state = StatusState::Pending;
                }
                CheckStatus::Pass | CheckStatus::Skipped => {
                    // Check review status
                    if self.pr_status.changes_required() {
                        status_message = "Changes required".to_string();
                        status_state = StatusState::Failure;
                    } else if !self.pr_status.mergeable && !self.pr_status.merged {
                        status_message = "Pull request is not mergeable.".to_string();
                        status_state = StatusState::Failure;
                    } else if !self.pr_status.missing_required_reviewers.is_empty() {
                        status_message = format!(
                            "Waiting on mandatory reviews ({})",
                            self.pr_status.missing_required_reviewers.join(", ")
                        );
                        status_state = StatusState::Pending;
                    } else if self.pr_status.needed_reviewers_count
                        > self.pr_status.approved_reviewers.len()
                    {
                        status_message = "Waiting on reviews".to_string();
                        status_state = StatusState::Pending;
                    } else {
                        // Check QA status
                        match self.pr_status.qa_status {
                            QaStatus::Fail => {
                                status_message = "QA failed. Please fix.".to_string();
                                status_state = StatusState::Failure;
                            }
                            QaStatus::Waiting => {
                                status_message = "Waiting for QA".to_string();
                                status_state = StatusState::Pending;
                            }
                            QaStatus::Pass | QaStatus::Skipped => {
                                if self.pr_status.locked {
                                    status_message = "PR is locked".to_string();
                                    status_state = StatusState::Failure;
                                }
                            }
                        }
                    }
                }
            }
        } else {
            status_message = "PR title does not match regex.".to_string();
            status_state = StatusState::Failure;
        }

        Ok(StatusMessage {
            state: status_state,
            title: status_title,
            message: status_message,
        })
    }
}

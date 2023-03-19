use github_scbot_domain_models::{ChecksStatus, QaStatus};
use github_scbot_ghapi_interface::types::GhCommitStatus;

use super::PullRequestStatus;
use crate::Result;

pub const VALIDATION_STATUS_MESSAGE: &str = "Validation";

pub struct StatusMessage {
    pub state: GhCommitStatus,
    pub title: &'static str,
    pub message: String,
}

pub struct GenerateStatusMessageUseCase<'a> {
    pub pr_status: &'a PullRequestStatus,
}

impl<'a> GenerateStatusMessageUseCase<'a> {
    #[tracing::instrument(skip(self))]
    pub fn run(&mut self) -> Result<StatusMessage> {
        let status_title = VALIDATION_STATUS_MESSAGE;
        let mut status_state = GhCommitStatus::Success;
        let mut status_message = "All good.".to_string();

        if self.pr_status.wip {
            status_message = "PR is still in WIP".to_string();
            status_state = GhCommitStatus::Failure;
        } else if self.pr_status.valid_pr_title {
            // Check CI status
            match self.pr_status.checks_status {
                ChecksStatus::Fail => {
                    status_message = "Checks failed. Please fix.".to_string();
                    status_state = GhCommitStatus::Failure;
                }
                ChecksStatus::Waiting => {
                    status_message = "Waiting for checks".to_string();
                    status_state = GhCommitStatus::Pending;
                }
                ChecksStatus::Pass | ChecksStatus::Skipped => {
                    // Check review status
                    if self.pr_status.changes_required() {
                        status_message = "Changes required".to_string();
                        status_state = GhCommitStatus::Failure;
                    } else if !self.pr_status.mergeable && !self.pr_status.merged {
                        status_message = "Pull request is not mergeable.".to_string();
                        status_state = GhCommitStatus::Failure;
                    } else if !self.pr_status.missing_required_reviewers.is_empty() {
                        status_message = format!(
                            "Waiting on mandatory reviews ({})",
                            self.pr_status.missing_required_reviewers.join(", ")
                        );
                        status_state = GhCommitStatus::Pending;
                    } else if self.pr_status.needed_reviewers_count
                        > self.pr_status.approved_reviewers.len()
                    {
                        status_message = "Waiting on reviews".to_string();
                        status_state = GhCommitStatus::Pending;
                    } else {
                        // Check QA status
                        match self.pr_status.qa_status {
                            QaStatus::Fail => {
                                status_message = "QA failed. Please fix.".to_string();
                                status_state = GhCommitStatus::Failure;
                            }
                            QaStatus::Waiting => {
                                status_message = "Waiting for QA".to_string();
                                status_state = GhCommitStatus::Pending;
                            }
                            QaStatus::Pass | QaStatus::Skipped => {
                                if self.pr_status.locked {
                                    status_message = "PR is locked".to_string();
                                    status_state = GhCommitStatus::Failure;
                                }
                            }
                        }
                    }
                }
            }
        } else {
            status_message = "PR title does not match regex.".to_string();
            status_state = GhCommitStatus::Failure;
        }

        Ok(StatusMessage {
            state: status_state,
            title: status_title,
            message: status_message,
        })
    }
}

use prbot_ghapi_interface::types::GhCommitStatusState;
use prbot_models::{ChecksStatus, QaStatus};

use super::PullRequestStatus;
use crate::Result;

pub const VALIDATION_STATUS_MESSAGE: &str = "Validation";

#[derive(Debug)]
pub struct StatusMessage {
    pub state: GhCommitStatusState,
    pub title: &'static str,
    pub message: String,
}

#[derive(Default)]
pub struct StatusMessageGenerator {
    _private: (),
}

impl StatusMessageGenerator {
    #[tracing::instrument(skip_all, ret)]
    pub fn generate(&self, pr_status: &PullRequestStatus) -> Result<StatusMessage> {
        let status_title = VALIDATION_STATUS_MESSAGE;
        let mut status_state = GhCommitStatusState::Success;
        let mut status_message = "All good.".to_string();

        if pr_status.wip {
            status_message = "PR is still in WIP".to_string();
            status_state = GhCommitStatusState::Failure;
        } else if pr_status.valid_pr_title {
            // Check CI status
            match pr_status.checks_status {
                ChecksStatus::Fail => {
                    status_message = "Checks failed. Please fix.".to_string();
                    status_state = GhCommitStatusState::Failure;
                }
                ChecksStatus::Waiting => {
                    status_message = "Waiting for checks".to_string();
                    status_state = GhCommitStatusState::Pending;
                }
                ChecksStatus::Pass | ChecksStatus::Skipped => {
                    // Check review status
                    if pr_status.changes_required() {
                        status_message = "Changes required".to_string();
                        status_state = GhCommitStatusState::Failure;
                    } else if !pr_status.mergeable && !pr_status.merged {
                        status_message = "Pull request is not mergeable.".to_string();
                        status_state = GhCommitStatusState::Failure;
                    } else if !pr_status.missing_required_reviewers.is_empty() {
                        status_message = format!(
                            "Waiting on mandatory reviews ({})",
                            pr_status.missing_required_reviewers.join(", ")
                        );
                        status_state = GhCommitStatusState::Pending;
                    } else if pr_status.needed_reviewers_count > pr_status.approved_reviewers.len()
                    {
                        status_message = "Waiting on reviews".to_string();
                        status_state = GhCommitStatusState::Pending;
                    } else {
                        // Check QA status
                        match pr_status.qa_status {
                            QaStatus::Fail => {
                                status_message = "QA failed. Please fix.".to_string();
                                status_state = GhCommitStatusState::Failure;
                            }
                            QaStatus::Waiting => {
                                status_message = "Waiting for QA".to_string();
                                status_state = GhCommitStatusState::Pending;
                            }
                            QaStatus::Pass | QaStatus::Skipped => {
                                if pr_status.locked {
                                    status_message = "PR is locked".to_string();
                                    status_state = GhCommitStatusState::Failure;
                                }
                            }
                        }
                    }
                }
            }
        } else {
            status_message = "PR title does not match regex.".to_string();
            status_state = GhCommitStatusState::Failure;
        }

        Ok(StatusMessage {
            state: status_state,
            title: status_title,
            message: status_message,
        })
    }
}

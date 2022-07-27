//! Status module.

mod pull_status;

use github_scbot_core::types::{
    labels::StepLabel,
    pulls::GhPullRequest,
    status::{CheckStatus, QaStatus, StatusState},
};
use github_scbot_database::DbService;
use github_scbot_ghapi::adapter::ApiService;
use github_scbot_redis::{LockStatus, RedisService};
pub use pull_status::PullRequestStatus;

use crate::{errors::Result, pulls::PullRequestLogic, summary::SummaryCommentSender};

const VALIDATION_STATUS_MESSAGE: &str = "Validation";

/// Status logic.
pub struct StatusLogic;

impl StatusLogic {
    /// Determine automatic step for a pull request.
    #[tracing::instrument(ret)]
    pub fn determine_automatic_step(pull_request_status: &PullRequestStatus) -> StepLabel {
        if pull_request_status.wip {
            StepLabel::Wip
        } else if pull_request_status.valid_pr_title {
            match pull_request_status.checks_status {
                CheckStatus::Pass | CheckStatus::Skipped => {
                    if pull_request_status.changes_required()
                        || !pull_request_status.mergeable && !pull_request_status.merged
                    {
                        StepLabel::AwaitingChanges
                    } else if pull_request_status.missing_required_reviews() {
                        StepLabel::AwaitingRequiredReview
                    } else if pull_request_status.missing_reviews() {
                        StepLabel::AwaitingReview
                    } else {
                        match pull_request_status.qa_status {
                            QaStatus::Fail => StepLabel::AwaitingChanges,
                            QaStatus::Waiting => StepLabel::AwaitingQa,
                            QaStatus::Pass | QaStatus::Skipped => {
                                if pull_request_status.locked {
                                    StepLabel::Locked
                                } else {
                                    StepLabel::AwaitingMerge
                                }
                            }
                        }
                    }
                }
                CheckStatus::Waiting => StepLabel::AwaitingChecks,
                CheckStatus::Fail => StepLabel::AwaitingChanges,
            }
        } else {
            StepLabel::AwaitingChanges
        }
    }

    /// Update pull request status.
    #[tracing::instrument(
        skip_all,
        fields(
            repo_owner = %repo_owner,
            repo_name = %repo_name,
            pr_number = pr_number,
            head_sha = %upstream_pr.head.sha
        )
    )]
    pub async fn update_pull_request_status(
        api_adapter: &dyn ApiService,
        db_adapter: &dyn DbService,
        redis_adapter: &dyn RedisService,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        upstream_pr: &GhPullRequest,
    ) -> Result<()> {
        let commit_sha = &upstream_pr.head.sha;
        let pr_status = PullRequestStatus::from_database(
            api_adapter,
            db_adapter,
            repo_owner,
            repo_name,
            pr_number,
            upstream_pr,
        )
        .await?;

        // Update step label.
        let step_label = Self::determine_automatic_step(&pr_status);
        PullRequestLogic::apply_pull_request_step(
            api_adapter,
            repo_owner,
            repo_name,
            pr_number,
            Some(step_label),
        )
        .await?;

        // Post status.
        SummaryCommentSender::create_or_update(
            api_adapter,
            db_adapter,
            redis_adapter,
            repo_owner,
            repo_name,
            pr_number,
            &pr_status,
        )
        .await?;

        // Create or update status.
        let (status_state, status_title, status_message) =
            Self::generate_pr_status_message(&pr_status)?;
        api_adapter
            .commit_statuses_update(
                repo_owner,
                repo_name,
                commit_sha,
                status_state,
                status_title,
                &status_message,
            )
            .await?;

        let pr_model = db_adapter
            .pull_requests()
            .get(repo_owner, repo_name, pr_number)
            .await?
            .unwrap();

        // Merge if auto-merge is enabled
        if matches!(step_label, StepLabel::AwaitingMerge)
            && upstream_pr.merged != Some(true)
            && pr_model.automerge()
        {
            // Use lock
            let key = format!("pr-merge_{}-{}_{}", repo_owner, repo_name, pr_number);
            if let LockStatus::SuccessfullyLocked(l) = redis_adapter.try_lock_resource(&key).await?
            {
                let result = PullRequestLogic::try_automerge_pull_request(
                    api_adapter,
                    db_adapter,
                    repo_owner,
                    repo_name,
                    pr_number,
                    upstream_pr,
                )
                .await?;
                if !result {
                    db_adapter
                        .pull_requests()
                        .set_automerge(repo_owner, repo_name, pr_number, false)
                        .await?;

                    // Update status
                    SummaryCommentSender::create_or_update(
                        api_adapter,
                        db_adapter,
                        redis_adapter,
                        repo_owner,
                        repo_name,
                        pr_number,
                        &pr_status,
                    )
                    .await?;
                }

                l.release().await?;
            }
        }

        Ok(())
    }

    /// Generate pull request status message.
    pub fn generate_pr_status_message(
        pull_request_status: &PullRequestStatus,
    ) -> Result<(StatusState, &'static str, String)> {
        let status_title = VALIDATION_STATUS_MESSAGE;
        let mut status_state = StatusState::Success;
        let mut status_message = "All good.".to_string();

        if pull_request_status.wip {
            status_message = "PR is still in WIP".to_string();
            status_state = StatusState::Failure;
        } else if pull_request_status.valid_pr_title {
            // Check CI status
            match pull_request_status.checks_status {
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
                    if pull_request_status.changes_required() {
                        status_message = "Changes required".to_string();
                        status_state = StatusState::Failure;
                    } else if !pull_request_status.mergeable && !pull_request_status.merged {
                        status_message = "Pull request is not mergeable.".to_string();
                        status_state = StatusState::Failure;
                    } else if !pull_request_status.missing_required_reviewers.is_empty() {
                        status_message = format!(
                            "Waiting on mandatory reviews ({})",
                            pull_request_status.missing_required_reviewers.join(", ")
                        );
                        status_state = StatusState::Pending;
                    } else if pull_request_status.needed_reviewers_count
                        > pull_request_status.approved_reviewers.len()
                    {
                        status_message = "Waiting on reviews".to_string();
                        status_state = StatusState::Pending;
                    } else {
                        // Check QA status
                        match pull_request_status.qa_status {
                            QaStatus::Fail => {
                                status_message = "QA failed. Please fix.".to_string();
                                status_state = StatusState::Failure;
                            }
                            QaStatus::Waiting => {
                                status_message = "Waiting for QA".to_string();
                                status_state = StatusState::Pending;
                            }
                            QaStatus::Pass | QaStatus::Skipped => {
                                if pull_request_status.locked {
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

        Ok((status_state, status_title, status_message))
    }

    /// Disable validation status.
    pub async fn disable_validation_status(
        api_adapter: &dyn ApiService,
        db_adapter: &dyn DbService,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<()> {
        let sha = api_adapter
            .pulls_get(repo_owner, repo_name, pr_number)
            .await?
            .head
            .sha;

        api_adapter
            .commit_statuses_update(
                repo_owner,
                repo_name,
                &sha,
                StatusState::Success,
                VALIDATION_STATUS_MESSAGE,
                "Bot disabled.",
            )
            .await?;

        SummaryCommentSender::delete(api_adapter, db_adapter, repo_owner, repo_name, pr_number)
            .await
    }
}

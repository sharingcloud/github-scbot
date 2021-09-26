//! Status module.

mod pull_status;

use github_scbot_api::adapter::IAPIAdapter;
use github_scbot_database::models::{IDatabaseAdapter, PullRequestModel, RepositoryModel};
use github_scbot_libs::tracing::debug;
use github_scbot_redis::{IRedisAdapter, LockStatus};
use github_scbot_types::{
    labels::StepLabel,
    status::{CheckStatus, QaStatus, StatusState},
};
pub use pull_status::PullRequestStatus;

use crate::{
    errors::Result,
    pulls::{apply_pull_request_step, try_automerge_pull_request},
    summary::SummaryCommentSender,
};

const VALIDATION_STATUS_MESSAGE: &str = "Validation";

/// Determine automatic step for a pull request.
pub fn determine_automatic_step(pull_request_status: &PullRequestStatus) -> Result<StepLabel> {
    Ok(if pull_request_status.wip {
        StepLabel::Wip
    } else if pull_request_status.valid_pr_title {
        match pull_request_status.checks_status {
            CheckStatus::Pass | CheckStatus::Skipped => {
                if pull_request_status.missing_required_reviews() {
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
    })
}

/// Create initial pull request status.
pub async fn create_initial_pull_request_status(
    api_adapter: &dyn IAPIAdapter,
    db_adapter: &dyn IDatabaseAdapter,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    commit_sha: &str,
) -> Result<()> {
    let pr_status = PullRequestStatus::from_database(db_adapter, repo_model, pr_model).await?;

    // Update step label.
    let step_label = determine_automatic_step(&pr_status)?;
    pr_model.set_step_label(step_label);
    db_adapter.pull_request().save(pr_model).await?;

    apply_pull_request_step(api_adapter, repo_model, pr_model).await?;

    // Create status comment
    SummaryCommentSender::new()
        .create(api_adapter, db_adapter, repo_model, pr_model)
        .await?;

    // Create or update status.
    let (status_state, status_title, status_message) = generate_pr_status_message(&pr_status)?;
    api_adapter
        .commit_statuses_update(
            &repo_model.owner,
            &repo_model.name,
            commit_sha,
            status_state,
            status_title,
            &status_message,
        )
        .await?;

    Ok(())
}

/// Update pull request status.
pub async fn update_pull_request_status(
    api_adapter: &dyn IAPIAdapter,
    db_adapter: &dyn IDatabaseAdapter,
    redis_adapter: &dyn IRedisAdapter,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    commit_sha: &str,
) -> Result<()> {
    let pr_status = PullRequestStatus::from_database(db_adapter, repo_model, pr_model).await?;

    // Update step label.
    let step_label = determine_automatic_step(&pr_status)?;
    pr_model.set_step_label(step_label);
    db_adapter.pull_request().save(pr_model).await?;

    apply_pull_request_step(api_adapter, repo_model, pr_model).await?;

    // Post status.
    SummaryCommentSender::new()
        .update(api_adapter, db_adapter, repo_model, pr_model)
        .await?;

    // Create or update status.
    let (status_state, status_title, status_message) = generate_pr_status_message(&pr_status)?;
    api_adapter
        .commit_statuses_update(
            &repo_model.owner,
            &repo_model.name,
            commit_sha,
            status_state,
            status_title,
            &status_message,
        )
        .await?;

    // Merge if auto-merge is enabled
    if matches!(step_label, StepLabel::AwaitingMerge) && !pr_model.merged && pr_model.automerge {
        // Use lock
        let key = format!(
            "pr-merge_{}-{}_{}",
            repo_model.owner,
            repo_model.name,
            pr_model.get_number()
        );
        if let LockStatus::SuccessfullyLocked(l) = redis_adapter.try_lock_resource(&key).await? {
            let result =
                try_automerge_pull_request(api_adapter, db_adapter, repo_model, pr_model).await?;
            if !result {
                pr_model.automerge = false;
                db_adapter.pull_request().save(pr_model).await?;

                // Update status
                SummaryCommentSender::new()
                    .update(api_adapter, db_adapter, repo_model, pr_model)
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

    debug!(
        pull_request_status = ?pull_request_status,
        message = "Generated pull request status"
    );

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
                if !pull_request_status.missing_required_reviewers.is_empty() {
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
    api_adapter: &dyn IAPIAdapter,
    db_adapter: &dyn IDatabaseAdapter,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
) -> Result<()> {
    let sha = api_adapter
        .pulls_get(&repo_model.owner, &repo_model.name, pr_model.get_number())
        .await?
        .head
        .sha;

    api_adapter
        .commit_statuses_update(
            &repo_model.owner,
            &repo_model.name,
            &sha,
            StatusState::Success,
            VALIDATION_STATUS_MESSAGE,
            "Bot disabled.",
        )
        .await?;

    SummaryCommentSender::new()
        .delete(api_adapter, db_adapter, repo_model, pr_model)
        .await
}

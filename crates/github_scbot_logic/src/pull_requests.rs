//! Pull requests logic.

use github_scbot_database::{
    models::{PullRequestModel, RepositoryModel, ReviewModel},
    DbConn,
};
use github_scbot_types::{
    common::GHUser,
    labels::StepLabel,
    pull_requests::{GHPullRequestAction, GHPullRequestEvent, GHPullRequestReviewState},
    status::{CheckStatus, QAStatus},
};

use crate::{
    database::process_pull_request,
    reviews::{handle_review_request, rerequest_existing_reviews},
    status::{update_pull_request_status, PullRequestStatus},
    welcome::post_welcome_comment,
    Result,
};

/// Handle GitHub pull request event.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `event` - GitHub pull request event
pub async fn handle_pull_request_event(conn: &DbConn, event: &GHPullRequestEvent) -> Result<()> {
    let (repo_model, mut pr_model) =
        process_pull_request(conn, &event.repository, &event.pull_request)?;

    // Welcome message
    if let GHPullRequestAction::Opened = event.action {
        post_welcome_comment(&repo_model, &pr_model, &event.pull_request.user.login).await?;
    }

    let mut status_changed = false;

    // Status update
    match event.action {
        GHPullRequestAction::Opened | GHPullRequestAction::Synchronize => {
            pr_model.wip = event.pull_request.draft;
            pr_model.set_checks_status(CheckStatus::Waiting);
            pr_model.save(conn)?;
            status_changed = true;

            // Only for synchronize
            rerequest_existing_reviews(conn, &repo_model, &pr_model).await?;
        }
        GHPullRequestAction::Reopened | GHPullRequestAction::ReadyForReview => {
            pr_model.wip = event.pull_request.draft;
            pr_model.save(conn)?;
            status_changed = true;
        }
        GHPullRequestAction::ConvertedToDraft => {
            pr_model.wip = true;
            pr_model.save(conn)?;
            status_changed = true;
        }
        GHPullRequestAction::ReviewRequested => {
            handle_review_request(
                conn,
                &pr_model,
                GHPullRequestReviewState::Pending,
                &extract_usernames(&event.pull_request.requested_reviewers),
            )?;
            status_changed = true;
        }
        GHPullRequestAction::ReviewRequestRemoved => {
            handle_review_request(
                conn,
                &pr_model,
                GHPullRequestReviewState::Dismissed,
                &extract_usernames(&event.pull_request.requested_reviewers),
            )?;
            status_changed = true;
        }
        _ => (),
    }

    if let GHPullRequestAction::Edited = event.action {
        // Update PR title
        pr_model.name = event.pull_request.title.clone();
        pr_model.save(conn)?;
        status_changed = true;
    }

    if status_changed {
        update_pull_request_status(
            conn,
            &repo_model,
            &mut pr_model,
            &event.pull_request.head.sha,
        )
        .await?;
    }

    Ok(())
}

/// Determine automatic step for a pull request.
///
/// # Arguments
///
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `reviews` - Reviews
pub fn determine_automatic_step(
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    reviews: &[ReviewModel],
) -> Result<StepLabel> {
    let status = PullRequestStatus::from_pull_request(repo_model, pr_model, reviews)?;

    Ok(if pr_model.wip {
        StepLabel::Wip
    } else {
        match pr_model.get_checks_status() {
            Some(CheckStatus::Pass) | Some(CheckStatus::Skipped) | None => {
                if status.missing_required_reviews() {
                    StepLabel::AwaitingRequiredReview
                } else if status.missing_reviews() {
                    StepLabel::AwaitingReview
                } else {
                    match status.qa_status {
                        Some(QAStatus::Fail) => StepLabel::AwaitingChanges,
                        Some(QAStatus::Waiting) | None => StepLabel::AwaitingQA,
                        Some(QAStatus::Pass) | Some(QAStatus::Skipped) => {
                            if status.locked {
                                StepLabel::Locked
                            } else {
                                StepLabel::AwaitingMerge
                            }
                        }
                    }
                }
            }
            Some(CheckStatus::Waiting) => StepLabel::AwaitingChecks,
            Some(CheckStatus::Fail) => StepLabel::AwaitingChanges,
        }
    })
}

fn extract_usernames(users: &[GHUser]) -> Vec<&str> {
    users.iter().map(|r| &r.login[..]).collect()
}

//! Pull requests logic.

use github_scbot_database::DbConn;
use github_scbot_types::{
    common::GHUser,
    pull_requests::{GHPullRequestAction, GHPullRequestEvent, GHPullRequestReviewState},
    status::CheckStatus,
};

use crate::{
    database::process_pull_request,
    reviews::{handle_review_request, rerequest_existing_reviews},
    status::update_pull_request_status,
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
            pr_model.set_step_auto();
            pr_model.save(conn)?;
            status_changed = true;

            // Only for synchronize
            rerequest_existing_reviews(conn, &pr_model)?;
        }
        GHPullRequestAction::Reopened | GHPullRequestAction::ReadyForReview => {
            pr_model.wip = event.pull_request.draft;
            pr_model.set_step_auto();
            pr_model.save(conn)?;
            status_changed = true;
        }
        GHPullRequestAction::ConvertedToDraft => {
            pr_model.wip = true;
            pr_model.set_step_auto();
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

fn extract_usernames(users: &[GHUser]) -> Vec<&str> {
    users.iter().map(|r| &r.login[..]).collect()
}

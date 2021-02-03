//! Pull requests webhook handlers.

use actix_web::HttpResponse;
use github_scbot_database::DbConn;
use github_scbot_logic::{
    database::process_pull_request, reviews::handle_review, status::update_pull_request_status,
    welcome::post_welcome_comment,
};
use github_scbot_types::{
    pull_requests::{
        GHPullRequestAction, GHPullRequestEvent, GHPullRequestReviewCommentEvent,
        GHPullRequestReviewEvent,
    },
    status::CheckStatus,
};
use tracing::info;

use crate::errors::Result;

pub(crate) async fn pull_request_event(
    conn: &DbConn,
    event: GHPullRequestEvent,
) -> Result<HttpResponse> {
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
        _ => (),
    }

    if let GHPullRequestAction::Edited = event.action {
        // Update PR title
        pr_model.name = event.pull_request.title;
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

    info!(
        "Pull request event from repository '{}', PR number #{}, action '{:?}' (from '{}')",
        event.repository.full_name,
        event.pull_request.number,
        event.action,
        event.pull_request.user.login
    );

    Ok(HttpResponse::Ok().body("Pull request."))
}

pub(crate) async fn pull_request_review_event(
    conn: &DbConn,
    event: GHPullRequestReviewEvent,
) -> Result<HttpResponse> {
    let (_repo, pr) = process_pull_request(conn, &event.repository, &event.pull_request)?;
    handle_review(conn, &pr, &event.review).await?;

    info!(
        "Pull request review event from repository '{}', PR number #{}, action '{:?}' (review from '{}')",
        event.repository.full_name, event.pull_request.number, event.action, event.review.user.login
    );

    Ok(HttpResponse::Ok().body("Pull request review."))
}

pub(crate) async fn pull_request_review_comment_event(
    conn: &DbConn,
    event: GHPullRequestReviewCommentEvent,
) -> Result<HttpResponse> {
    process_pull_request(conn, &event.repository, &event.pull_request)?;

    info!(
        "Pull request review comment event from repository '{}', PR number #{}, action '{:?}' (comment from '{}')",
        event.repository.full_name, event.pull_request.number, event.action, event.comment.user.login
    );

    Ok(HttpResponse::Ok().body("Pull request review comment."))
}

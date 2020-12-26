//! Webhook pull request handlers

use actix_web::HttpResponse;
use tracing::info;

use crate::api::status::update_status_for_repo;
use crate::database::models::{CheckStatus, DbConn};
use crate::webhook::errors::Result;
use crate::webhook::logic::{
    database::{apply_pull_request_step, process_pull_request},
    status::{generate_pr_status, post_status_comment},
    welcome::post_welcome_comment,
};
use crate::webhook::types::{
    PullRequestAction, PullRequestEvent, PullRequestReviewCommentEvent, PullRequestReviewEvent,
};

pub async fn pull_request_event(conn: &DbConn, event: PullRequestEvent) -> Result<HttpResponse> {
    let (repo_model, mut pr_model) =
        process_pull_request(conn, &event.repository, &event.pull_request)?;

    // Welcome message
    if let PullRequestAction::Opened = event.action {
        post_welcome_comment(&repo_model, &pr_model, &event.pull_request.user.login).await?;
    }

    let mut status_changed = false;

    // Status update
    match event.action {
        PullRequestAction::Opened | PullRequestAction::Synchronize => {
            pr_model.update_wip(conn, event.pull_request.draft)?;
            pr_model.update_check_status(conn, Some(CheckStatus::Waiting))?;
            pr_model.update_step_auto(conn)?;
            status_changed = true;
        }
        PullRequestAction::Reopened | PullRequestAction::ReadyForReview => {
            pr_model.update_wip(conn, event.pull_request.draft)?;
            pr_model.update_step_auto(conn)?;
            status_changed = true;
        }
        PullRequestAction::ConvertedToDraft => {
            pr_model.update_wip(conn, true)?;
            pr_model.update_step_auto(conn)?;
            status_changed = true;
        }
        _ => (),
    }

    if let PullRequestAction::Edited = event.action {
        // Update PR title
        pr_model.update_name(conn, &event.pull_request.title)?;
        status_changed = true;
    }

    if status_changed {
        apply_pull_request_step(&repo_model, &pr_model).await?;
        post_status_comment(conn, &repo_model, &mut pr_model).await?;

        // Create or update status
        let (status_state, status_title, status_message) =
            generate_pr_status(&repo_model, &pr_model)?;
        update_status_for_repo(
            &repo_model,
            &event.pull_request.head.sha,
            status_state,
            status_title,
            status_message,
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

pub async fn pull_request_review_event(
    conn: &DbConn,
    event: PullRequestReviewEvent,
) -> Result<HttpResponse> {
    process_pull_request(conn, &event.repository, &event.pull_request)?;

    info!(
        "Pull request review event from repository '{}', PR number #{}, action '{:?}' (review from '{}')",
        event.repository.full_name, event.pull_request.number, event.action, event.review.user.login
    );

    Ok(HttpResponse::Ok().body("Pull request review."))
}

pub async fn pull_request_review_comment_event(
    conn: &DbConn,
    event: PullRequestReviewCommentEvent,
) -> Result<HttpResponse> {
    process_pull_request(conn, &event.repository, &event.pull_request)?;

    info!(
        "Pull request review comment event from repository '{}', PR number #{}, action '{:?}' (comment from '{}')",
        event.repository.full_name, event.pull_request.number, event.action, event.comment.user.login
    );

    Ok(HttpResponse::Ok().body("Pull request review comment."))
}

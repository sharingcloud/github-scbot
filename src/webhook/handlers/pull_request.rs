//! Webhook pull request handlers

use actix_web::HttpResponse;
use eyre::Result;
use log::info;

use crate::database::models::{CheckStatus, DbConn};
use crate::webhook::logic::{
    apply_pull_request_step, post_status_comment, post_welcome_comment, process_pull_request,
};
use crate::webhook::types::{
    PullRequestAction, PullRequestEvent, PullRequestReviewCommentEvent, PullRequestReviewEvent,
};

pub async fn pull_request_event(conn: &DbConn, event: PullRequestEvent) -> Result<HttpResponse> {
    let (repo_model, mut pr_model) =
        process_pull_request(conn, &event.repository, &event.pull_request)?;

    if let PullRequestAction::Opened = event.action {
        post_welcome_comment(&repo_model, &pr_model, &event.pull_request.user.login).await?;
    }

    if let PullRequestAction::Synchronize = event.action {
        // Reset status check
        pr_model.update_check_status(conn, Some(CheckStatus::Waiting))?;
    }

    let comment_id = post_status_comment(&repo_model, &pr_model).await?;
    apply_pull_request_step(&repo_model, &pr_model).await?;
    pr_model.update_status_comment(conn, comment_id)?;

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

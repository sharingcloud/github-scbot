//! Webhook pull request handlers

use std::convert::TryInto;

use actix_web::HttpResponse;
use diesel::prelude::*;
use eyre::Result;
use log::info;

use crate::api::comments::{create_or_update_status_comment, post_welcome_comment};
use crate::database::models::{CheckStatus, DbConn, PullRequestModel};
use crate::webhook::logic::process_pull_request;
use crate::webhook::types::{
    PullRequestAction, PullRequestEvent, PullRequestReviewCommentEvent, PullRequestReviewEvent,
};

pub async fn pull_request_event(conn: &DbConn, event: PullRequestEvent) -> Result<HttpResponse> {
    let (repo_model, mut pr_model) =
        process_pull_request(conn, &event.repository, &event.pull_request)?;

    if let PullRequestAction::Opened = event.action {
        post_welcome_comment(
            &event.repository.owner.login,
            &event.repository.name,
            event.pull_request.number,
            &event.pull_request.user.login,
        )
        .await?;
    }

    if let PullRequestAction::Synchronize = event.action {
        // Reset status check
        pr_model.check_status = CheckStatus::Waiting.as_str().to_string();
        pr_model.save_changes::<PullRequestModel>(conn)?;
    }

    let comment_id = create_or_update_status_comment(&repo_model, &pr_model).await?;
    pr_model.status_comment_id = comment_id.try_into()?;
    pr_model.save_changes::<PullRequestModel>(conn)?;

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

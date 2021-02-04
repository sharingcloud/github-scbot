//! Pull requests webhook handlers.

use actix_web::HttpResponse;
use github_scbot_database::DbConn;
use github_scbot_logic::{
    database::process_pull_request, pull_requests::handle_pull_request_event,
    reviews::handle_review,
};
use github_scbot_types::pull_requests::{
    GHPullRequestEvent, GHPullRequestReviewCommentEvent, GHPullRequestReviewEvent,
};
use tracing::info;

use crate::errors::Result;

pub(crate) async fn pull_request_event(
    conn: &DbConn,
    event: GHPullRequestEvent,
) -> Result<HttpResponse> {
    info!(
        "Pull request event from repository '{}', PR number #{}, action '{:?}' (from '{}')",
        event.repository.full_name,
        event.pull_request.number,
        event.action,
        event.pull_request.user.login
    );

    handle_pull_request_event(conn, &event).await?;
    Ok(HttpResponse::Ok().body("Pull request."))
}

pub(crate) async fn pull_request_review_event(
    conn: &DbConn,
    event: GHPullRequestReviewEvent,
) -> Result<HttpResponse> {
    info!(
        "Pull request review event from repository '{}', PR number #{}, action '{:?}' (review from '{}')",
        event.repository.full_name, event.pull_request.number, event.action, event.review.user.login
    );

    let (_repo, pr) = process_pull_request(conn, &event.repository, &event.pull_request)?;
    handle_review(conn, &pr, &event.review).await?;
    Ok(HttpResponse::Ok().body("Pull request review."))
}

pub(crate) async fn pull_request_review_comment_event(
    conn: &DbConn,
    event: GHPullRequestReviewCommentEvent,
) -> Result<HttpResponse> {
    info!(
        "Pull request review comment event from repository '{}', PR number #{}, action '{:?}' (comment from '{}')",
        event.repository.full_name, event.pull_request.number, event.action, event.comment.user.login
    );

    process_pull_request(conn, &event.repository, &event.pull_request)?;
    Ok(HttpResponse::Ok().body("Pull request review comment."))
}

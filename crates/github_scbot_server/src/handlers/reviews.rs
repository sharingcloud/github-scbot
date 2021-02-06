//! Review webhook handlers.

use actix_web::HttpResponse;
use github_scbot_database::DbConn;
use github_scbot_logic::{database::process_pull_request, reviews::handle_review_event};
use github_scbot_types::reviews::{GHReviewCommentEvent, GHReviewEvent};
use tracing::info;

use crate::errors::Result;

pub(crate) async fn review_event(conn: &DbConn, event: GHReviewEvent) -> Result<HttpResponse> {
    info!(
        "Pull request review event from repository '{}', PR number #{}, action '{:?}' (review from '{}')",
        event.repository.full_name, event.pull_request.number, event.action, event.review.user.login
    );

    handle_review_event(conn, &event).await?;
    Ok(HttpResponse::Ok().body("Pull request review."))
}

pub(crate) async fn review_comment_event(
    conn: &DbConn,
    event: GHReviewCommentEvent,
) -> Result<HttpResponse> {
    info!(
        "Pull request review comment event from repository '{}', PR number #{}, action '{:?}' (comment from '{}')",
        event.repository.full_name, event.pull_request.number, event.action, event.comment.user.login
    );

    process_pull_request(conn, &event.repository, &event.pull_request)?;
    Ok(HttpResponse::Ok().body("Pull request review comment."))
}

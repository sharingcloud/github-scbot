//! Review webhook handlers.

use actix_web::HttpResponse;
use github_scbot_conf::Config;
use github_scbot_database::DbPool;
use github_scbot_logic::reviews::handle_review_event;
use github_scbot_types::{events::EventType, reviews::GhReviewEvent};
use tracing::info;

use super::parse_event_type;
use crate::errors::Result;

pub(crate) fn parse_review_event(body: &str) -> Result<GhReviewEvent> {
    parse_event_type(EventType::PullRequestReview, body)
}

pub(crate) async fn review_event(
    config: Config,
    pool: DbPool,
    event: GhReviewEvent,
) -> Result<HttpResponse> {
    info!(
        "Pull request review event from repository '{}', PR number #{}, action '{:?}' (review from '{}')",
        event.repository.full_name, event.pull_request.number, event.action, event.review.user.login
    );

    handle_review_event(config, pool, event).await?;
    Ok(HttpResponse::Ok().body("Pull request review."))
}

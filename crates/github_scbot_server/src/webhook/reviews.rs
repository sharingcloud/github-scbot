//! Review webhook handlers.

use actix_web::HttpResponse;
use github_scbot_api::adapter::IAPIAdapter;
use github_scbot_conf::Config;
use github_scbot_database::models::IDatabaseAdapter;
use github_scbot_logic::reviews::handle_review_event;
use github_scbot_redis::IRedisAdapter;
use github_scbot_types::{events::EventType, reviews::GhReviewEvent};
use tracing::info;

use super::parse_event_type;
use crate::errors::Result;

pub(crate) fn parse_review_event(body: &str) -> Result<GhReviewEvent> {
    parse_event_type(EventType::PullRequestReview, body)
}

pub(crate) async fn review_event(
    config: &Config,
    api_adapter: &dyn IAPIAdapter,
    db_adapter: &dyn IDatabaseAdapter,
    redis_adapter: &dyn IRedisAdapter,
    event: GhReviewEvent,
) -> Result<HttpResponse> {
    info!(
        repository_path = %event.repository.full_name,
        pull_request_number = event.pull_request.number,
        action = ?event.action,
        review_author = %event.review.user.login,
        message = "Pull request review event",
    );

    handle_review_event(config, api_adapter, db_adapter, redis_adapter, event).await?;
    Ok(HttpResponse::Ok().body("Pull request review."))
}

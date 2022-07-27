//! Review webhook handlers.

use actix_web::HttpResponse;
use github_scbot_core::types::{events::EventType, reviews::GhReviewEvent};
use github_scbot_database2::DbService;
use github_scbot_ghapi::adapter::ApiService;
use github_scbot_logic::reviews::handle_review_event;
use github_scbot_redis::RedisService;
use snafu::ResultExt;

use super::parse_event_type;
use crate::errors::LogicSnafu;
use crate::errors::Result;

pub(crate) fn parse_review_event(body: &str) -> Result<GhReviewEvent> {
    parse_event_type(EventType::PullRequestReview, body)
}

pub(crate) async fn review_event(
    api_adapter: &dyn ApiService,
    db_adapter: &dyn DbService,
    redis_adapter: &dyn RedisService,
    event: GhReviewEvent,
) -> Result<HttpResponse> {
    handle_review_event(api_adapter, db_adapter, redis_adapter, event)
        .await
        .context(LogicSnafu)?;
    Ok(HttpResponse::Ok().body("Pull request review."))
}

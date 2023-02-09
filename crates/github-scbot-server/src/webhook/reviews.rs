//! Review webhook handlers.

use actix_web::HttpResponse;
use github_scbot_core::types::{events::EventType, reviews::GhReviewEvent};
use github_scbot_database_interface::DbService;
use github_scbot_domain::use_cases::reviews::HandleReviewEventUseCase;
use github_scbot_ghapi_interface::ApiService;
use github_scbot_lock_interface::LockService;

use super::parse_event_type;
use crate::{Result, ServerError};

pub(crate) fn parse_review_event(body: &str) -> Result<GhReviewEvent> {
    parse_event_type(EventType::PullRequestReview, body)
}

pub(crate) async fn review_event(
    api_adapter: &dyn ApiService,
    db_adapter: &mut dyn DbService,
    redis_adapter: &dyn LockService,
    event: GhReviewEvent,
) -> Result<HttpResponse> {
    HandleReviewEventUseCase {
        api_service: api_adapter,
        db_service: db_adapter,
        redis_service: redis_adapter,
        event,
    }
    .run()
    .await
    .map_err(|e| ServerError::DomainError { source: e })?;
    Ok(HttpResponse::Ok().body("Pull request review."))
}

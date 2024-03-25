//! Review webhook handlers.

use std::sync::Arc;

use actix_web::HttpResponse;
use prbot_core::use_cases::reviews::HandleReviewEventInterface;
use prbot_ghapi_interface::types::GhReviewEvent;
use shaku::HasComponent;

use super::parse_event_type;
use crate::{event_type::EventType, server::AppContext, Result, ServerError};

pub(crate) fn parse_review_event(body: &str) -> Result<GhReviewEvent> {
    parse_event_type(EventType::PullRequestReview, body)
}

pub(crate) async fn review_event(
    ctx: Arc<AppContext>,
    event: GhReviewEvent,
) -> Result<HttpResponse> {
    tokio::spawn(async move {
        let ctx = ctx.as_core_context();
        let handle_review_event: &dyn HandleReviewEventInterface = ctx.core_module.resolve_ref();
        handle_review_event
            .run(&ctx, event)
            .await
            .map_err(|e| ServerError::DomainError { source: e })
            .unwrap();
    });

    Ok(HttpResponse::Accepted().body("Pull request review."))
}
